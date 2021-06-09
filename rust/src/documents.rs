use crate::{
    methods::{decode::decode, encode::encode},
    pubsub::publish,
    utils::{schemas, uuids},
};
use defaults::Defaults;
use eyre::{bail, Result};
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    collections::{hash_map::Entry, HashMap},
    env, fs,
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, TryRecvError},
        Arc, Mutex, MutexGuard,
    },
};
use stencila_schema::Node;
use strum::ToString;

#[derive(Debug, JsonSchema, Serialize, ToString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[schemars(deny_unknown_fields)]
enum DocumentEventType {
    Deleted,
    Renamed,
    Modified,
    Encoded,
}

#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct DocumentEvent {
    /// The type of event
    #[serde(rename = "type")]
    type_: DocumentEventType,

    /// The document associated with the event
    #[schemars(schema_with = "DocumentEvent::schema_document")]
    document: Document,

    /// The content associated with the event, only provided for, `modified`
    /// and `encoded` events.
    content: Option<String>,

    /// The format of the document, only provided for, `modified` (the format
    /// of the document) and `encoded` events (the format of the encoding).
    format: Option<String>,
}

impl DocumentEvent {
    /// Generate the JSON Schema for the `document` property to avoid nesting
    fn schema_document(_generator: &mut SchemaGenerator) -> Schema {
        schemas::typescript("Document", true)
    }
}

/// The status of a document with respect to on-disk synchronization
#[derive(Debug, Clone, JsonSchema, Serialize, ToString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
enum DocumentStatus {
    /// The document `content` is the same as on disk at its `path`.
    Synced,
    /// The document `content` has modifications that have not yet
    /// been written to its `path`.
    Unwritten,
    /// The document `path` has modifications that have not yet
    /// been read into its `content`.
    Unread,
    /// The document `path` no longer exists and is now set to `None`.
    /// The user will need to choose a new path for the document if they
    /// want to save it.
    Deleted,
}

/// An in-memory representation of a document
#[derive(Debug, Clone, JsonSchema, Defaults, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Document {
    /// The document identifier
    pub id: String,

    /// The absolute path of the document's file.
    pub path: PathBuf,

    /// Whether or not the document's file is in the temporary
    /// directory.
    temporary: bool,

    /// The synchronization status of the document.
    /// This is orthogonal to `temporary` because a document's
    /// `content` can be synced or un-synced with the file system
    /// regardless of whether or not its `path` is temporary..
    #[def = "DocumentStatus::Unread"]
    status: DocumentStatus,

    /// The name of the document
    ///
    /// Usually the filename from the `path` but "Unnamed"
    /// for temporary documents.
    name: String,

    /// The current content of the document.
    ///
    /// When a `new()` document is created, the `content` will be open.
    /// When a document is `read()` from a file the `content` is the content
    /// of the file. The `content` may subsequently be changed using
    /// the `load()` function. A call to `write()` will write the content
    /// back to `path`.
    #[serde(skip)]
    content: String,

    /// The format of the document's `content`.
    ///
    /// On initialization, this is inferred, if possible, from the file name extension
    /// of the document's `path`. However, it may change whilst the document is
    /// open in memory (e.g. if the `load` function sets a different format).
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,

    /// The root Stencila Schema node of the document
    #[serde(skip)]
    root: Option<Node>,

    /// The set of unique subscriptions to this document
    ///
    /// Keeps track of the number of subscribers to each of the document's
    /// topic channels. Events will only be published on channels that
    /// have at least one subscriber.
    ///
    /// Valid subscription topics are the names of the `DocumentEvent` types:
    ///
    /// - `removed`: published when document file is deleted
    /// - `renamed`: published when document file is renamed
    /// - `modified`: published when document file is modified
    /// - `encoded:<format>` published when a document's content
    ///   is changed internally or externally and  conversions have been
    ///   completed e.g. `encoded:html`
    subscriptions: HashMap<String, u32>,
}

impl Document {
    /// Create a new empty document.
    ///
    /// # Arguments
    ///
    /// - `format`: The format of the document.
    ///
    /// This function is intended to be used by editors when creating
    /// a new document. The created document will be `temporary: true`
    /// and have a temporary file path.
    fn new(format: Option<String>) -> Document {
        let path = env::temp_dir().join(uuids::generate(uuids::Family::File));
        // Ensure that the file exists
        if !path.exists() {
            fs::write(path.clone(), "").expect("Unable to write temporary file");
        }

        let id = uuids::generate(uuids::Family::Document);

        Document {
            id,
            path,
            temporary: true,
            status: DocumentStatus::Synced,
            name: "Unnamed".into(),
            format,
            ..Default::default()
        }
    }

    /// Open a document from an existing file.
    ///
    /// # Arguments
    ///
    /// - `path`: the path of the file to create the document from
    ///
    /// - `format`: The format of the document. If `None` will be inferred from
    ///             the file extension.
    async fn open(path: PathBuf, format: Option<String>) -> Result<Document> {
        if path.is_dir() {
            bail!("Can not open a folder as a document; maybe try opening it as a project instead.")
        }

        let id = uuids::generate(uuids::Family::Document);

        let name = path
            .file_name()
            .map(|os_str| os_str.to_string_lossy())
            .unwrap_or_else(|| "Untitled".into())
            .into();

        let format = format.or_else(|| {
            path.extension()
                .map(|ext| ext.to_string_lossy().to_lowercase())
        });

        let mut document = Document {
            id,
            path,
            temporary: false,
            name,
            format,
            ..Default::default()
        };
        document.read().await?;

        Ok(document)
    }

    /// Read the document from the file system and return its content.
    ///
    /// Sets `status` to `Synced`.
    async fn read(&mut self) -> Result<String> {
        let content = fs::read_to_string(&self.path)?;
        self.load(content, None).await?;
        self.status = DocumentStatus::Synced;
        Ok(self.content.clone())
    }

    /// Write the document to the file system, optionally load new `content`
    /// and set `format` before doing so.
    ///
    /// # Arguments
    ///
    /// - `content`: the content to load into the document
    /// - `format`: the format of the content; if not supplied assumed to be
    ///    the document's existing format.
    ///
    /// Sets `status` to `Synced`.
    async fn write(&mut self, content: Option<String>, format: Option<String>) -> Result<()> {
        if let Some(content) = content {
            self.load(content, format).await?;
        }

        fs::write(&self.path, self.content.as_bytes())?;
        self.status = DocumentStatus::Synced;

        Ok(())
    }

    /// Save the document to another file, possibly in another format
    ///
    /// # Arguments
    ///
    /// - `path`: the path for the new file.
    /// - `format`: the format to dump the content as; if not supplied assumed to be
    ///    the document's existing format.
    ///
    /// Note: this does not change the `path` or `format` of the current
    /// document.
    #[cfg(ignore)]
    fn save_as(&self, path: &str, format: Option<String>) -> Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(self.dump(format)?.as_bytes())?;
        Ok(())
    }

    /// Dump the document's content to a string in its current, or
    /// a different, format
    ///
    /// # Arguments
    ///
    /// - `format`: the format to dump the content as; if not supplied assumed to be
    ///    the document's existing format.
    async fn dump(&self, format: Option<String>) -> Result<String> {
        let format = match format {
            Some(format) => format,
            None => return Ok(self.content.clone()),
        };
        if let Some(root) = &self.root {
            return encode(root, &format).await;
        }
        bail!("Document has no root node")
    }

    /// Load content into the document
    ///
    /// # Arguments
    ///
    /// - `content`: the content to load into the document
    /// - `format`: the format of the content; if not supplied assumed to be
    ///    the document's existing format.
    ///
    /// Publishes `encoded:` events for each of the formats subscribed to.
    /// Sets `status` to `Unwritten`.
    /// In the future, this will also trigger an `import()` to convert the `content`
    /// into a Stencila `CreativeWork` nodes and set the document's `root` (from which
    /// the conversions will be done).
    async fn load(&mut self, content: String, format: Option<String>) -> Result<()> {
        // Set the `content` and `status` of the document
        self.content = content;
        self.status = DocumentStatus::Unwritten;
        if let Some(format) = format {
            self.format = Some(format)
        }

        // To decode the content we need to know, or assume, its format
        let format = match &self.format {
            Some(format) => format.as_str(),
            None => "txt",
        };

        // Import the content into the `root` node of the document
        let root = decode(&self.content, format).await?;

        // Encode the `root` node into each of the formats for which there are subscriptions
        for subscription in self.subscriptions.keys() {
            if let Some(format) = subscription.strip_prefix("encoded:") {
                match encode(&root, format).await {
                    Ok(content) => {
                        self.publish(
                            DocumentEventType::Encoded,
                            Some(content),
                            Some(format.into()),
                        );
                    }
                    Err(error) => {
                        tracing::warn!("Unable to encode to format \"{}\": {}", format, error)
                    }
                }
            }
        }

        // Now that we're done borrowing the root node for encoding to
        // different formats, store it.
        self.root = Some(root);

        Ok(())
    }

    /// Add a subscriber to one of the document's topics
    #[allow(clippy::unnecessary_wraps)]
    fn subscribe(&mut self, topic: &str) -> Result<()> {
        match self.subscriptions.entry(topic.into()) {
            Entry::Occupied(mut entry) => {
                let subscribers = entry.get_mut();
                *subscribers += 1;
            }
            Entry::Vacant(entry) => {
                entry.insert(1);
            }
        }
        Ok(())
    }

    /// Query the document
    ///
    /// Returns a JSON value. Returns `null` if the query does not select anything.
    fn query(&self, query: &str, lang: &str) -> Result<serde_json::Value> {
        let result = match lang {
            "jmespath" => {
                let expr = jmespatch::compile(query)?;
                let result = expr.search(&self.root)?;
                let result = serde_json::to_value(result.clone())?;
                result
            }
            "jsonptr" => {
                let data = serde_json::to_value(&self.root)?;
                let result = data.pointer(query);
                match result {
                    Some(value) => value.clone(),
                    None => serde_json::Value::Null,
                }
            }
            _ => bail!("Unknown query language '{}'", lang),
        };
        Ok(result)
    }

    /// Remove a subscriber to one of the document's topics
    #[allow(clippy::unnecessary_wraps)]
    fn unsubscribe(&mut self, topic: &str) -> Result<()> {
        match self.subscriptions.entry(topic.into()) {
            Entry::Occupied(mut entry) => {
                let subscribers = entry.get_mut();
                *subscribers -= 1;
                if *subscribers == 0 {
                    entry.remove();
                }
            }
            Entry::Vacant(_) => {}
        }
        Ok(())
    }

    /// Publish a `DocumentEvent` for this document
    fn publish(&self, type_: DocumentEventType, content: Option<String>, format: Option<String>) {
        let topic = match type_ {
            DocumentEventType::Encoded => format!("encoded:{}", format.clone().unwrap_or_default()),
            _ => type_.to_string(),
        };
        let topic = format!("documents:{}:{}", self.id, topic);

        publish(
            &topic,
            &DocumentEvent {
                type_,
                document: self.clone(),
                content,
                format,
            },
        )
    }

    /// Called when the file is removed from the file system
    ///
    /// Sets `status` to `Deleted` and publishes a `Deleted` event so that,
    /// for example, a document's tab can be updated to indicate it is deleted.
    fn deleted(&mut self, path: PathBuf) {
        tracing::debug!("Document removed: {}", path.display());

        self.status = DocumentStatus::Deleted;

        self.publish(DocumentEventType::Deleted, None, None)
    }

    /// Called when the file is renamed
    ///
    /// Changes the `path` and publishes a `Renamed` event so that, for example,
    /// a document's tab can be updated with the new file name.
    fn renamed(&mut self, from: PathBuf, to: PathBuf) {
        tracing::debug!("Document renamed: {} to {}", from.display(), to.display());

        self.path = to;

        self.publish(DocumentEventType::Renamed, None, None)
    }

    /// Called when the file is modified
    ///
    /// Reads the file into `content` and emits a `Modified` event so that the user
    /// can be asked if they want to load the new content into editor, or overwrite with
    /// existing editor content.
    async fn modified(&mut self, path: PathBuf) {
        tracing::debug!("Document modified: {}", path.display());

        self.status = DocumentStatus::Unread;

        match self.read().await {
            Ok(content) => self.publish(
                DocumentEventType::Modified,
                Some(content),
                self.format.clone(),
            ),
            Err(error) => tracing::error!("While attempting to read modified file: {}", error),
        }
    }
}

#[derive(Debug, Clone)]
struct DocumentWatcher {
    sender: std::sync::mpsc::Sender<()>,
}

impl DocumentWatcher {
    fn new(path: PathBuf, document: Arc<Mutex<Document>>) -> DocumentWatcher {
        let (thread_sender, thread_receiver) = channel();
        std::thread::spawn(move || -> Result<()> {
            use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
            use std::time::Duration;

            let (watcher_sender, watcher_receiver) = channel();
            let mut watcher = watcher(watcher_sender, Duration::from_millis(100))?;
            watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();

            let path = path.display().to_string();
            let span = tracing::info_span!("document_watch", path = path.as_str());
            let _enter = span.enter();
            tracing::debug!("Starting document file watch: {}", path);

            let lock = || document.lock().expect("Unable to lock document");

            let handle = |event| match event {
                DebouncedEvent::Remove(path) => lock().deleted(path),
                DebouncedEvent::Rename(from, to) => lock().renamed(from, to),
                // TODO: reinstate once this is async
                //DebouncedEvent::Write(path) => lock().modified(path).await,
                _ => {}
            };

            loop {
                if let Err(TryRecvError::Disconnected) = thread_receiver.try_recv() {
                    tracing::debug!("Ending document file watch: {}", path);
                    break;
                }
                match watcher_receiver.recv() {
                    Ok(event) => handle(event),
                    Err(error) => tracing::error!("Watch error: {:?}", error),
                }
            }

            Ok(())
        });
        DocumentWatcher {
            sender: thread_sender,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentHandler {
    #[serde(flatten)]
    document: Arc<Mutex<Document>>,

    #[serde(skip)]
    watcher: DocumentWatcher,
}

impl DocumentHandler {
    /// Create a new document handler.
    ///
    /// # Arguments
    ///
    /// - `format`: The format of the document.
    fn new(document: Document) -> DocumentHandler {
        let path = document.path.clone();
        let document: Arc<Mutex<Document>> = Arc::new(Mutex::new(document));
        let watcher = DocumentWatcher::new(path, Arc::clone(&document));
        DocumentHandler { document, watcher }
    }
}

/// An in-memory store of documents
#[derive(Clone, Debug, Default)]
pub struct Documents {
    /// A mapping of file paths to open documents
    registry: HashMap<String, DocumentHandler>,
}

impl Documents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> Result<Vec<String>> {
        Ok(self.registry.keys().cloned().collect::<Vec<String>>())
    }

    pub fn create(&mut self, format: Option<String>) -> Result<Document> {
        let document = Document::new(format);
        self.registry
            .insert(document.id.clone(), DocumentHandler::new(document.clone()));
        Ok(document)
    }

    pub async fn open(&mut self, path: &str, format: Option<String>) -> Result<Document> {
        let path = Path::new(path).canonicalize()?;

        for handler in self.registry.values() {
            let document = handler.document.lock().expect("Unable to lock document");
            if document.path == path {
                return Ok(document.clone());
            }
        }

        let document = Document::open(path, format).await?;
        self.registry
            .insert(document.id.clone(), DocumentHandler::new(document.clone()));
        Ok(document)
    }

    pub fn get(&mut self, id: &str) -> Result<MutexGuard<Document>> {
        if let Some(handle) = self.registry.get(id) {
            if let Ok(guard) = handle.document.lock() {
                Ok(guard)
            } else {
                bail!("Unable to lock document {}", id)
            }
        } else {
            bail!("No document with id {}", id)
        }
    }

    pub async fn read(&mut self, id: &str) -> Result<String> {
        self.get(&id)?.read().await
    }

    pub async fn write(&mut self, id: &str, content: Option<String>) -> Result<()> {
        self.get(&id)?.write(content, None).await
    }

    pub async fn dump(&mut self, id: &str, format: Option<String>) -> Result<String> {
        self.get(&id)?.dump(format).await
    }

    pub async fn load(&mut self, id: &str, content: String) -> Result<()> {
        self.get(&id)?.load(content, None).await
    }

    pub fn subscribe(&mut self, id: &str, topic: &str) -> Result<()> {
        self.get(&id)?.subscribe(topic)
    }

    pub fn unsubscribe(&mut self, id: &str, topic: &str) -> Result<()> {
        self.get(&id)?.unsubscribe(topic)
    }

    pub fn close(&mut self, id: &str) -> Result<()> {
        self.registry.remove(id);
        Ok(())
    }
}

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![
        schemas::generate::<Document>()?,
        schemas::generate::<DocumentEvent>()?,
    ]);
    Ok(schemas)
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use crate::cli::display;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage documents",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub struct Command {
        #[structopt(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub enum Action {
        List(List),
        Open(Open),
        Close(Close),
        Show(Show),
        Query(Query),
        Convert(Convert),
        Schemas(Schemas),
    }

    impl Command {
        pub async fn run(&self, documents: &mut Documents) -> display::Result {
            let Self { action } = self;
            match action {
                Action::List(action) => action.run(documents),
                Action::Open(action) => action.run(documents).await,
                Action::Close(action) => action.run(documents),
                Action::Show(action) => action.run(documents).await,
                Action::Query(action) => action.run(documents).await,
                Action::Convert(action) => action.run(documents).await,
                Action::Schemas(action) => action.run(),
            }
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "List open documents >",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}

    impl List {
        pub fn run(&self, documents: &mut Documents) -> display::Result {
            let list = documents.list()?;
            display::value(list)
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Open a document >",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Open {
        /// The path of the document file
        #[structopt(default_value = ".")]
        pub file: String,
    }

    impl Open {
        pub async fn run(&self, documents: &mut Documents) -> display::Result {
            let Self { file } = self;
            documents.open(file, None).await?;
            display::nothing()
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Close a document >",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Close {
        /// The path of the document file
        #[structopt(default_value = ".")]
        pub file: String,
    }

    impl Close {
        pub fn run(&self, documents: &mut Documents) -> display::Result {
            let Self { file } = self;
            documents.close(file)?;
            display::nothing()
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Show a document",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// The path of the document file
        pub file: String,

        /// The format of the file
        #[structopt(short, long)]
        format: Option<String>,
    }

    impl Show {
        pub async fn run(&self, documents: &mut Documents) -> display::Result {
            let Self { file, format } = self;
            let document = documents.open(file, format.clone()).await?;
            display::value(document)
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Show a document",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Query {
        /// The path of the document file
        file: String,

        /// The query to run on the document
        query: String,

        /// The format of the file
        #[structopt(short, long)]
        format: Option<String>,

        /// The language of the query
        #[structopt(
            short,
            long,
            default_value = "jmespath",
            possible_values = &QUERY_LANGS
        )]
        lang: String,
    }

    const QUERY_LANGS: [&str; 2] = ["jmespath", "jsonptr"];

    impl Query {
        pub async fn run(&self, documents: &mut Documents) -> display::Result {
            let Self {
                file,
                format,
                query,
                lang,
            } = self;
            let document = documents.open(file, format.clone()).await?;
            let result = document.query(query, lang)?;
            display::value(result)
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Convert a document to another format",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Convert {
        /// The path of the input document
        pub input: String,

        /// The path of the output document
        pub output: String,

        /// The format of the input (defaults to being inferred from the file extension or content type)
        #[structopt(short, long)]
        from: Option<String>,

        /// The format of the output (defaults to being inferred from the file extension)
        #[structopt(short, long)]
        to: Option<String>,
    }

    impl Convert {
        pub async fn run(&self, documents: &mut Documents) -> display::Result {
            let Self {
                input,
                output,
                from,
                to,
            } = self;
            let _document = documents.open(input, from.clone()).await?;
            todo!("convert to {} as format {:?}", output, to);
            #[allow(unreachable_code)]
            display::nothing()
        }
    }
    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Get JSON Schemas for documents and associated types",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Schemas {}

    impl Schemas {
        pub fn run(&self) -> display::Result {
            let schema = schemas()?;
            display::value(schema)
        }
    }
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;

    use super::*;

    #[test]
    fn document_new() {
        let doc = Document::new(None);
        assert!(doc.path.starts_with(env::temp_dir()));
        assert!(doc.temporary);
        assert!(matches!(doc.status, DocumentStatus::Synced));
        assert!(doc.format.is_none());
        assert_eq!(doc.content, "");
        assert!(doc.root.is_none());
        assert_eq!(doc.subscriptions, hashmap! {});

        let doc = Document::new(Some("md".to_string()));
        assert!(doc.path.starts_with(env::temp_dir()));
        assert!(doc.temporary);
        assert!(matches!(doc.status, DocumentStatus::Synced));
        assert_eq!(doc.format.unwrap(), "md");
        assert_eq!(doc.content, "");
        assert!(doc.root.is_none());
        assert_eq!(doc.subscriptions, hashmap! {});
    }

    #[tokio::test]
    async fn document_open() -> Result<()> {
        let fixtures = &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("fixtures")
            .join("articles");

        for file in vec!["elife-small.json", "elife-mid.json", "era-plotly.json"] {
            let doc = Document::open(fixtures.join(file), None).await?;
            assert!(doc.path.starts_with(fixtures));
            assert!(!doc.temporary);
            assert!(matches!(doc.status, DocumentStatus::Synced));
            assert_eq!(doc.format.unwrap(), "json");
            assert!(doc.content.len() > 0);
            assert!(doc.root.is_some());
            assert_eq!(doc.subscriptions, hashmap! {});
        }

        Ok(())
    }
}
