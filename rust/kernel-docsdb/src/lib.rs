use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
};

use dirs::{closest_stencila_dir, stencila_db_dir, stencila_store_dir};
use kernel_kuzu::{
    kernel::{
        common::{
            async_trait::async_trait,
            eyre::{bail, Result},
            itertools::Itertools,
            once_cell::sync::Lazy,
            regex::Regex,
            serde_json,
            tokio::{
                self,
                fs::read_to_string,
                sync::{watch, Mutex},
            },
            tracing,
        },
        format::Format,
        generate_id,
        schema::{
            get, Array, Excerpt, ExecutionBounds, ExecutionMessage, Node, NodeId, NodeSet,
            Primitive, Reference, SoftwareApplication,
        },
        Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder,
    },
    KuzuKernelInstance,
};
use lru::LruCache;
use node_db::NodeDatabase;

pub use kernel_kuzu::QueryResultTransform;

const NAME: &str = "docsdb";

/// A kernel for querying Stencila node databases
///
/// This kernel allows for querying of Stencila node databases using Cypher
/// Query language. It extends the `KuzuKernel` with these key differences:
///
/// 1. execution bounds are always `Box` i.e. read-only and node filesystem access
///
/// 2. special comments which allow access to in-memory, local, and remote databases
///
///   - `// @current` : the current document
///   - `// @workspace` : the current workspace (i.e. the closes `./stencila/db` directory)
///   - in the future remote database with other collections of documents
///
/// 3. returns nodes as `Excerpt`s (rather than as Cytoscape graph specs)
#[derive(Default)]
pub struct DocsDBKernel;

impl Kernel for DocsDBKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Database
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Cypher]
    }

    fn supported_bounds(&self) -> Vec<ExecutionBounds> {
        vec![ExecutionBounds::Box]
    }

    fn supports_variable_requests(&self) -> bool {
        true
    }

    fn create_instance(&self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(DocsDBKernelInstance::new(None, None)?))
    }
}

#[derive(Debug)]
enum DocsDB {
    Workspace,
    Document,
}

#[derive(Debug)]
pub struct DocsDBKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// Which database is active
    which_db: DocsDB,

    /// The path that the kernel is started in
    ///
    /// Used to determine the closest `.stencila` directory if necessary.
    directory: Option<PathBuf>,

    /// The Kuzu kernel instance used to query the workspace database
    workspace_db: KuzuKernelInstance,

    /// The document storage directory associated with the database
    store: Option<PathBuf>,

    /// A document LRU cache to avoid repeated deserialization of documents
    cache: Option<Mutex<LruCache<String, Node>>>,

    /// The Kuzu kernel instance used to query the document database
    document_db: KuzuKernelInstance,

    /// The document that nodes will be retrieved from
    ///
    /// The boolean indicates whether the current value has been
    /// synced with the in-memory database
    document: Option<Arc<Mutex<(Node, bool)>>>,
}

impl DocsDBKernelInstance {
    /// Create a new instance
    pub fn new(
        directory: Option<PathBuf>,
        doc_receiver: Option<watch::Receiver<Node>>,
    ) -> Result<Self> {
        let id = generate_id(NAME);
        let workspace_db = KuzuKernelInstance::box_with(id.clone(), QueryResultTransform::Excerpts);
        let document_db = KuzuKernelInstance::box_with(id.clone(), QueryResultTransform::Excerpts);

        let document = if let Some(mut receiver) = doc_receiver {
            let node = receiver.borrow().clone();
            let document = Arc::new(Mutex::new((node, false)));

            {
                let document = document.clone();
                tokio::task::spawn(async move {
                    while receiver.changed().await.is_ok() {
                        let node = receiver.borrow_and_update().clone();
                        *document.lock().await = (node, false);
                    }
                });
            }

            Some(document)
        } else {
            None
        };

        Ok(Self {
            id,
            which_db: DocsDB::Workspace,
            directory,
            workspace_db,
            store: None,
            cache: None,
            document_db,
            document,
        })
    }

    /// Create a new instance for the current document
    pub fn new_document(receiver: watch::Receiver<Node>) -> Result<Self> {
        let mut instance = Self::new(None, Some(receiver))?;
        instance.use_document()?;
        Ok(instance)
    }

    /// Create a new instance for the workspace associated with a path
    pub async fn new_workspace(path: &Path) -> Result<Self> {
        let mut instance = Self::new(Some(path.into()), None)?;
        instance.use_workspace().await?;
        Ok(instance)
    }

    /// Use the document database if available
    fn use_document(&mut self) -> Result<()> {
        if self.document.is_none() {
            bail!("No document associated with this kernel")
        }

        self.which_db = DocsDB::Document;

        Ok(())
    }

    /// Use the workspace database associated with a path
    ///
    /// Finds the closest `.stencila` directory and uses its `.stencila/db`
    /// and `.stencila/store` subdirectories.
    async fn use_workspace(&mut self) -> Result<()> {
        let home_dir = self.directory.clone().unwrap_or_else(|| PathBuf::from("."));
        let stencila_dir = closest_stencila_dir(&home_dir, false).await?;

        let db_dir = stencila_db_dir(&stencila_dir, false).await?;
        self.workspace_db.set_path(db_dir);

        let store_dir = stencila_store_dir(&stencila_dir, false).await?;
        self.store = Some(store_dir);
        if let Some(cache) = self.cache.as_mut() {
            cache.lock().await.clear();
        } else {
            self.cache = Some(Mutex::new(LruCache::new(
                NonZeroUsize::new(10).expect("valid non-zero"),
            )));
        }

        self.which_db = DocsDB::Workspace;

        Ok(())
    }

    /// Create Stencila [`Excerpt`]s from an [`Array`] of doc ids and node paths
    async fn excerpts_from_array(&self, array: &Array) -> Result<Vec<Node>> {
        let mut excerpts = Vec::new();
        for item in &array.0 {
            let Primitive::String(pair) = item else {
                bail!("Expected string")
            };

            let Some((doc_id, node_path_str, node_ancestors, node_type)) =
                pair.split(":").collect_tuple()
            else {
                bail!("Expected : separator")
            };

            let node_path = node_path_str.parse()?;

            let (source, excerpt) = match self.which_db {
                DocsDB::Workspace => {
                    let (Some(store), Some(cache)) = (&self.store, &self.cache) else {
                        bail!("Store and cache expected for workspace use")
                    };

                    let mut cache = cache.lock().await;
                    match cache.get(doc_id) {
                        Some(doc) => {
                            let source = Reference::from(doc);
                            let excerpt = get(doc, node_path);

                            (source, excerpt)
                        }
                        None => {
                            let path = store.join(format!("{doc_id}.json"));
                            let json = read_to_string(path).await?;
                            let doc = serde_json::from_str(&json)?;

                            let source = Reference::from(&doc);
                            let excerpt = get(&doc, node_path);

                            cache.put(doc_id.to_string(), doc);

                            (source, excerpt)
                        }
                    }
                }
                DocsDB::Document => {
                    let Some(doc) = &self.document else {
                        bail!("Document expected")
                    };

                    let (doc, ..) = &*doc.lock().await;

                    let source = Reference {
                        title: Some("Current document".into()),
                        ..Default::default()
                    };

                    let excerpt = get(doc, node_path);

                    (source, excerpt)
                }
            };

            let Ok(node) = excerpt else {
                tracing::warn!("Unable to find node path in `{doc_id}`");
                continue;
            };

            let node = match node {
                NodeSet::One(node) => node,
                NodeSet::Many(nodes) => {
                    tracing::warn!("Unexpected `NodeSet::Many`");
                    match nodes.into_iter().next() {
                        Some(node) => node,
                        None => continue,
                    }
                }
            };

            let content = if node.node_type().is_block() {
                // If the node is a block, then just use it as the content
                // of the excerpt
                vec![node.try_into()?]
            } else {
                // If the node is not a block (e.g. Article, TableRow, ListItem) then
                // attempt to convert to a vector of blocks
                match node.try_into() {
                    Ok(block) => block,
                    Err(error) => {
                        tracing::warn!("While converting to blocks: {error}");
                        continue;
                    }
                }
            };

            let excerpt = Node::Excerpt(Excerpt {
                source,
                node_path: node_path_str.to_string(),
                node_ancestors: node_ancestors.to_string(),
                node_type: node_type.to_string(),
                content,
                ..Default::default()
            });

            excerpts.push(excerpt)
        }

        Ok(excerpts)
    }
}

#[async_trait]
impl KernelInstance for DocsDBKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        tracing::trace!("Starting DocsDB kernel");

        self.directory = Some(directory.to_path_buf());

        self.workspace_db.start(directory).await
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing query in DocsDB kernel");

        // Check for db aliases and set db and store paths accordingly
        static DB_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^\/\/\s*@(document|workspace)\s*$").expect("invalid regex"));
        let mut lines = Vec::new();
        for line in code.lines() {
            if let Some(captures) = DB_REGEX.captures(line) {
                let alias = &captures[1];
                match alias {
                    "document" => self.use_document()?,
                    "workspace" => self.use_workspace().await?,
                    _ => unreachable!(),
                }
                // Add comment line back to retain correct line numbering for any errors
                lines.push("//");
            } else {
                lines.push(line);
            }
        }

        let kuzu_kernel = match self.which_db {
            DocsDB::Workspace => &mut self.workspace_db,
            DocsDB::Document => {
                if let Some(node) = &self.document {
                    let (node, synced) = &mut *node.lock().await;

                    // Update the database with the document if not done so yet
                    if !*synced {
                        let database = self.document_db.database()?;
                        let mut db = NodeDatabase::attached(database)?;
                        let doc_id = NodeId::new(b"doc", &[0]);
                        db.upsert(&doc_id, &node)?;
                        *synced = true;
                    }
                }
                &mut self.document_db
            }
        };

        // Execute the code
        let (mut outputs, messages) = kuzu_kernel.execute(&lines.join("\n")).await?;

        // If the output is an array of excerpt doc ids and node paths then hydrate them into nodes
        if let (1, Some(Node::Array(excerpts))) = (outputs.len(), outputs.first()) {
            if let Some(Primitive::String(excerpt)) = excerpts.first() {
                if excerpt.starts_with("doc_") && excerpt.contains(":") {
                    outputs = self.excerpts_from_array(excerpts).await?;
                }
            }
        }

        Ok((outputs, messages))
    }

    async fn set(&mut self, name: &str, value: &Node) -> Result<()> {
        self.workspace_db.set(name, value).await
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        self.workspace_db.get(name).await
    }

    fn variable_channel(
        &mut self,
        requester: KernelVariableRequester,
        responder: KernelVariableResponder,
    ) {
        self.workspace_db.variable_channel(requester, responder)
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting DocsDB kernel info");

        Ok(SoftwareApplication {
            name: "DocsDB Kernel".to_string(),
            ..self.workspace_db.info().await?
        })
    }

    async fn replicate(&mut self, bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        tracing::trace!("Replicating DocsDB kernel");

        self.workspace_db.replicate(bounds).await
    }
}
