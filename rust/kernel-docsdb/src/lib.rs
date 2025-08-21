use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use lru::LruCache;

use codecs::EncodeOptions;
use dirs::{closest_stencila_dir, stencila_db_file, stencila_store_dir};
use kernel_kuzu::{
    KuzuKernelInstance,
    kernel::{
        Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder,
        common::{
            async_trait::async_trait,
            eyre::{Result, bail},
            futures::future::join_all,
            itertools::Itertools,
            once_cell::sync::Lazy,
            regex::Regex,
            serde_json, serde_yaml,
            tokio::{
                self,
                fs::read_to_string,
                sync::{Mutex, mpsc, oneshot, watch},
                task,
            },
            tracing,
        },
        format::Format,
        generate_id,
        schema::{
            Array, Excerpt, ExecutionBounds, ExecutionMessage, Node, NodeId, NodeSet, NodeType,
            Primitive, Reference, SoftwareApplication, Variable, get, shortcuts::t,
        },
    },
};
use node_canonicalize::canonicalize;
use node_db::NodeDatabase;
use node_sentencize::sentencize;

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
        Ok(Box::new(DocsDBKernelInstance::new(None, None, None)?))
    }
}

#[derive(Debug)]
enum DocsDB {
    Workspace,
    Document,
}

pub type DocsDBRootReceiver = watch::Receiver<Node>;

// Channel for requesting a list of variables from kernels
pub type DocsDBVariableListReceiver = mpsc::Receiver<(String, oneshot::Sender<Vec<Variable>>)>;
pub type DocsDBVariableListSender = mpsc::Sender<(String, oneshot::Sender<Vec<Variable>>)>;

pub type DocsDBChannels = (DocsDBRootReceiver, DocsDBVariableListSender);

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

    /// A channel sender to request a list of variables from kernels
    variables_requester: Option<DocsDBVariableListSender>,

    /// The most recently fetched list of variables
    variables: Option<Vec<Variable>>,
}

impl DocsDBKernelInstance {
    /// Create a new instance
    pub fn new(
        directory: Option<PathBuf>,
        channels: Option<DocsDBChannels>,
        id: Option<String>,
    ) -> Result<Self> {
        let id = id.unwrap_or_else(|| generate_id(NAME));
        let workspace_db =
            KuzuKernelInstance::main_with(id.clone(), QueryResultTransform::Excerpts);
        let document_db = KuzuKernelInstance::box_with(id.clone(), QueryResultTransform::Excerpts);

        let (document, variables_requester) =
            if let Some((mut root_receiver, variables_requester)) = channels {
                let node = root_receiver.borrow().clone();
                let document = Arc::new(Mutex::new((node, false)));

                {
                    let document = document.clone();
                    tokio::task::spawn(async move {
                        while root_receiver.changed().await.is_ok() {
                            let node = root_receiver.borrow_and_update().clone();
                            *document.lock().await = (node, false);
                        }
                    });
                }

                (Some(document), Some(variables_requester))
            } else {
                (None, None)
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
            variables_requester,
            variables: None,
        })
    }

    /// Create a new instance for the current document
    pub fn new_document(id: &str, channels: DocsDBChannels) -> Result<Self> {
        let mut instance = Self::new(None, Some(channels), Some(id.into()))?;
        instance.use_document()?;
        Ok(instance)
    }

    /// Create a new instance for the workspace associated with a path
    pub async fn new_workspace(id: &str, path: &Path) -> Result<Self> {
        let mut instance = Self::new(Some(path.into()), None, Some(id.into()))?;
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

        let db_path = stencila_db_file(&stencila_dir, false).await?;
        if !db_path.exists() {
            NodeDatabase::new(&db_path)?;
        }
        self.workspace_db.set_path(db_path);

        let store_dir = stencila_store_dir(&stencila_dir, false).await?;
        if !store_dir.exists() {
            bail!(
                "Workspace store `{}` does not exist yet",
                store_dir.display()
            )
        }
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

            let Some((doc_id, node_id, node_path_str, node_ancestors, node_type, position)) =
                pair.split(":").collect_tuple()
            else {
                bail!("Expected : separator")
            };

            if node_type == "Variable" {
                let excerpt = self.excerpt_from_variable(node_id).await?;
                excerpts.push(excerpt);
                continue;
            }

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
                        title: Some(vec![t("Current document")]),
                        ..Default::default()
                    };

                    let excerpt = get(doc, node_path);

                    (source, excerpt)
                }
            };

            // Generate a unique but deterministic id for the excerpt
            let doi = if let Some(doi) = &source.doi {
                // Replaces characters which may be in DOI which may interfere with
                // with Markdown parsing of citations
                doi.to_string().replace(['@', ';', '[', ']'], "-")
            } else {
                ["10.0000/", &doc_id[4..]].concat()
            };
            let id = Some([&doi, "#", position].concat());

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
                id,
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

    /// Create a Stencila [`Excerpt`] for a [`Variable`]
    async fn excerpt_from_variable(&self, node_id: &str) -> Result<Node> {
        let Some(variables) = &self.variables else {
            bail!("Variables expected")
        };

        let node_id = NodeId::from_str(node_id)?;

        let mut variable = None;
        for var in variables {
            if var.node_id() == node_id {
                variable = Some(var);
                break;
            }
        }

        let Some(variable) = variable else {
            bail!("Variable not found");
        };

        let mut title = String::new();
        if let Some(lang) = &variable.programming_language {
            title += lang;
            title += " variable ";
        } else {
            title += "Variable ";
        }
        title += &variable.name;

        let source = Reference {
            title: Some(vec![t(title)]),
            ..Default::default()
        };

        let mut md = format!("Variable `{}`", variable.name);

        if let Some(type_) = variable
            .native_type
            .as_ref()
            .or(variable.node_type.as_ref())
        {
            md.push_str(&format!(" is a `{type_}`"));
        }

        if let Some(lang) = &variable.programming_language {
            md.push_str(" defined in ");
            md.push_str(lang);
        }

        md.push_str(". ");

        if let Some(native_hint) = variable.native_hint.as_ref() {
            md.push_str(native_hint);
        } else if let Some(hint) = variable.hint.as_ref() {
            let yaml = serde_yaml::to_string(hint)?;
            md.push_str(&format!(
                " A summary of the variable:\n\n```yaml\n{yaml}\n```"
            ));
        };

        let content = codec_markdown::decode(&md, None)?.0.try_into()?;

        Ok(Node::Excerpt(Excerpt {
            source,
            node_path: String::new(),
            node_ancestors: "Document".to_string(),
            node_type: NodeType::Variable.to_string(),
            content,
            ..Default::default()
        }))
    }

    /// Add documents to the database from identifiers
    ///
    /// For each document (which may have multiple identifiers), tries each
    /// identifier in order until one succeeds. Returns the number of documents
    /// successfully added. Uses parallel processing for efficiency.
    pub async fn add_documents(&mut self, documents: &[Vec<String>]) -> Result<usize> {
        let Ok(mut db) = self
            .workspace_db
            .database()
            .and_then(NodeDatabase::attached)
        else {
            bail!("No workspace database")
        };

        let Some(store_dir) = &self.store else {
            bail!("No store directory for workspace database")
        };

        // Process documents in parallel
        let store_dir_clone = store_dir.clone();
        let tasks = documents.iter().map(|document_identifiers| {
            let store_dir = store_dir_clone.clone();
            let identifiers = document_identifiers.clone();

            task::spawn(async move {
                // Try each identifier for this document until one succeeds
                for identifier in identifiers {
                    match codecs::from_identifier(&identifier, None).await {
                        Ok(mut root) => {
                            // Generate a unique document ID
                            let doc_id = NodeId::random(*b"doc");

                            // Canonicalize and sentencize the document
                            if let Err(error) = canonicalize(&mut root).await {
                                tracing::debug!(
                                    "Failed to canonicalize document `{identifier}`: {error}",
                                );
                                continue;
                            }
                            sentencize(&mut root);

                            // Store the document as JSON
                            let store_path = store_dir.join(format!("{doc_id}.json"));
                            if let Err(error) = codec_json::to_path(
                                &root,
                                &store_path,
                                Some(EncodeOptions {
                                    compact: Some(false),
                                    ..Default::default()
                                }),
                            ) {
                                tracing::debug!(
                                    "Failed to store document from `{identifier}`: {error}",
                                );
                                continue;
                            }

                            tracing::debug!(
                                "Successfully processed document from identifier: {identifier}",
                            );

                            return Some((doc_id, root));
                        }
                        Err(error) => {
                            tracing::debug!(
                                "Failed to load document from identifier {identifier}: {error}"
                            );
                        }
                    }
                }

                tracing::debug!("Failed to add document using any of its identifiers");
                None
            })
        });

        // Wait for all tasks to complete
        let results = join_all(tasks).await;

        // Insert successful documents into database
        let mut added_count = 0;
        for result in results {
            match result {
                Ok(Some((doc_id, root))) => {
                    // Upsert to database
                    db.upsert(&doc_id, &root)?;
                    added_count += 1;
                }
                Ok(None) => {
                    // Document failed to load from any identifier
                }
                Err(e) => {
                    tracing::error!("Task failed: {}", e);
                }
            }
        }

        if added_count == 0 {
            bail!(
                "Failed to add any documents. This may be due to network connectivity issues or unsupported identifiers."
            );
        }

        Ok(added_count)
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

    #[tracing::instrument(skip(self))]
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
                let Some(node) = &self.document else {
                    bail!("No document attached to document database")
                };
                let (node, synced) = &mut *node.lock().await;

                let doc_id = NodeId::new(b"doc", b"0");

                // Update the database with the document root node, if out of sync
                if !*synced {
                    tracing::debug!("Updating document nodes");

                    let database = self.document_db.database()?;
                    let mut db = NodeDatabase::attached(database)?;
                    db.upsert(&doc_id, node)?;
                    *synced = true;
                }

                // If the query potentially involves variables, update the variables list
                if let (Some(requester), true) = (
                    &self.variables_requester,
                    code.to_lowercase().contains("variable"),
                ) {
                    tracing::debug!("Updating document variables");

                    let (response_sender, response_receiver) = oneshot::channel();
                    requester
                        .send((self.id.to_string(), response_sender))
                        .await?;

                    if let Ok(variables) = response_receiver.await {
                        let database = self.document_db.database()?;
                        let mut db = NodeDatabase::attached(database)?;
                        db.delete_all("Variable")?;
                        db.insert_associated(&doc_id, &variables)?;
                        self.variables = Some(variables);
                    }
                }

                &mut self.document_db
            }
        };

        // Execute the code
        let (mut outputs, messages) = kuzu_kernel.execute(&lines.join("\n")).await?;

        // If the output is an array of excerpt doc ids and node paths then hydrate them into nodes
        if let (1, Some(Node::Array(excerpts))) = (outputs.len(), outputs.first())
            && let Some(Primitive::String(excerpt)) = excerpts.first()
            && excerpt.starts_with("doc_")
            && excerpt.contains(":")
        {
            outputs = self.excerpts_from_array(excerpts).await?;
        }

        Ok((outputs, messages))
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        if let Some(var) = self.workspace_db.get(name).await? {
            return Ok(Some(var));
        }

        if let Some(var) = self.document_db.get(name).await? {
            return Ok(Some(var));
        }

        Ok(None)
    }

    async fn set(&mut self, name: &str, value: &Node) -> Result<()> {
        self.workspace_db.set(name, value).await?;
        self.document_db.set(name, value).await
    }

    async fn remove(&mut self, name: &str) -> Result<()> {
        self.workspace_db.remove(name).await?;
        self.document_db.remove(name).await
    }

    fn variable_channel(
        &mut self,
        requester: KernelVariableRequester,
        responder: KernelVariableResponder,
    ) {
        self.workspace_db
            .variable_channel(requester.clone(), responder.resubscribe());
        self.document_db.variable_channel(requester, responder)
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

        self.workspace_db.replicate(bounds).await?;
        self.document_db.replicate(bounds).await
    }
}
