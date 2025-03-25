use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use dirs::{closest_stencila_dir, stencila_db_dir, stencila_store_dir};
use kernel_kuzu::{
    KuzuKernelInstance, QueryResultTransform,
    kernel::{
        Kernel, KernelInstance, KernelType, KernelVariableRequester, KernelVariableResponder,
        common::{
            async_trait::async_trait,
            eyre::{Result, bail},
            itertools::Itertools,
            once_cell::sync::Lazy,
            regex::Regex,
            serde_json,
            tokio::{fs::read_to_string, sync::Mutex},
            tracing,
        },
        format::Format,
        generate_id,
        schema::{
            Array, Article, CreativeWorkType, Excerpt, ExecutionBounds, ExecutionMessage, Node,
            Primitive, SoftwareApplication, duplicate,
        },
    },
};
use lru::LruCache;

const NAME: &str = "docs";

/// A kernel for querying Stencila node databases
///
/// This kernel allows for querying of Stencila node databases using Cypher
/// Query language. It extends the `KuzuKernel` with these key differences:
///
/// 1. execution bounds are always `Box` i.e. read-only and node filesystem access
///
/// 2. special comments which allow access to in-memory, local, and remote databases
///
///   - `//current` : the current document
///   - `//workspace` : the current workspace (i.e. the closes `./stencila/db` directory)
///   - in the future remote database with other collections of documents
///
/// 3. returns nodes as `Excerpt`s (rather than as Cytoscape graph specs)
#[derive(Default)]
pub struct DocsKernel;

impl Kernel for DocsKernel {
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
        Ok(Box::new(DocsKernelInstance::new()))
    }
}

pub struct DocsKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The Kuzu kernel instance used to query the database
    kuzu: KuzuKernelInstance,

    /// The path that the kernel is started in
    ///
    /// Used to determine the closest `.stencila` directory if necessary.
    directory: Option<PathBuf>,

    /// The document storage directory associated with the database
    store: Option<PathBuf>,

    /// A document LRU cache to avoid repeated deserialization of the same document
    cache: Mutex<LruCache<String, Node>>,
}

impl DocsKernelInstance {
    /// Create a new instance
    fn new() -> Self {
        let id = generate_id(NAME);
        let kuzu = KuzuKernelInstance::box_with(id.clone(), QueryResultTransform::Excerpts);

        let docs = Mutex::new(LruCache::new(
            NonZeroUsize::new(10).expect("valid non-zero"),
        ));

        Self {
            kuzu,
            id,
            directory: None,
            store: None,
            cache: docs,
        }
    }

    /// Set/reset the store path and clear the documents cache
    async fn set_store(&mut self, path: PathBuf) {
        self.store = Some(path);
        self.cache.lock().await.clear();
    }

    /// Create Stencila [`Excerpt`]s from an [`Array`] of doc ids and node paths
    async fn excerpts_from_array(&self, array: &Array) -> Result<Vec<Node>> {
        let Some(store) = &self.store else {
            bail!("Expected store to be available");
        };

        let mut excerpts = Vec::new();
        for item in &array.0 {
            let Primitive::String(pair) = item else {
                bail!("Expected string")
            };

            let Some((doc_id, node_path)) = pair.split(":").collect_tuple() else {
                bail!("Expected : separator")
            };

            let node_path = node_path.parse()?;

            let (source, excerpt) = {
                let mut docs = self.cache.lock().await;
                match docs.get(doc_id) {
                    Some(doc) => {
                        // TODO: add a cite_as function to cite doc
                        let source = CreativeWorkType::Article(Article::default());
                        let excerpt = duplicate(doc, node_path);

                        (source, excerpt)
                    }
                    None => {
                        let path = store.join(format!("{doc_id}.json"));
                        let json = read_to_string(path).await?;
                        let doc = serde_json::from_str(&json)?;

                        // TODO: add a cite_as function to cite doc
                        let source = CreativeWorkType::Article(Article::default());
                        let excerpt = duplicate(&doc, node_path);

                        docs.put(doc_id.to_string(), doc);

                        (source, excerpt)
                    }
                }
            };

            let Ok(node) = excerpt else {
                tracing::warn!("Unable to find node path in `{doc_id}`");
                continue;
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

            let excerpt = Node::Excerpt(Excerpt::new(Box::new(source), content));

            excerpts.push(excerpt)
        }

        Ok(excerpts)
    }
}

#[async_trait]
impl KernelInstance for DocsKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        tracing::trace!("Starting docs kernel");

        self.directory = Some(directory.to_path_buf());

        self.kuzu.start(directory).await
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing query in docs kernel");

        // Check for db aliases and set db and store paths accordingly
        static DB_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^\/\/\s*db\s+(workspace)\s*$").expect("invalid regex"));
        let mut lines = Vec::new();
        for line in code.lines() {
            if let Some(captures) = DB_REGEX.captures(line) {
                let alias = &captures[1];
                match alias {
                    "workspace" => {
                        let current_dir =
                            self.directory.clone().unwrap_or_else(|| PathBuf::from("."));
                        let stencila_dir = closest_stencila_dir(&current_dir, false).await?;

                        let db_dir = stencila_db_dir(&stencila_dir, false).await?;
                        self.kuzu.set_path(db_dir);

                        let store_dir = stencila_store_dir(&stencila_dir, false).await?;
                        self.set_store(store_dir).await;
                    }
                    _ => unreachable!(),
                }
                // Add comment line back to retain correct line numbering for any errors
                lines.push("//");
            } else {
                lines.push(line);
            }
        }

        // Execute the code
        let (mut outputs, messages) = self.kuzu.execute(&lines.join("\n")).await?;

        // If the output is an array of excerpt paths then hydrate them into nodes
        if let (1, Some(Node::Array(excerpts))) = (outputs.len(), outputs.first()) {
            outputs = self.excerpts_from_array(excerpts).await?;
        }

        Ok((outputs, messages))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting docs kernel info");

        Ok(SoftwareApplication {
            name: "Docs Kernel".to_string(),
            ..self.kuzu.info().await?
        })
    }

    fn variable_channel(
        &mut self,
        requester: KernelVariableRequester,
        responder: KernelVariableResponder,
    ) {
        self.kuzu.variable_channel(requester, responder)
    }

    async fn replicate(&mut self, bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        tracing::trace!("Replicating docs kernel");

        self.kuzu.replicate(bounds).await
    }
}
