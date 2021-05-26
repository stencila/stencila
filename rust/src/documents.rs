use crate::{pubsub::publish, schemas, uuids};
use defaults::Defaults;
use eyre::{bail, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    collections::{hash_map::Entry, HashMap},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        mpsc::{channel, TryRecvError},
        Arc, Mutex, MutexGuard,
    },
};
use stencila_schema::CreativeWorkTypes;
use strum::{EnumVariantNames, ToString};

#[derive(JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct DocumentRemovedEvent {}

#[derive(JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct DocumentRenamedEvent {
    to: PathBuf,
}

#[derive(JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct DocumentModifiedEvent {
    content: String,
}

#[derive(JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct DocumentEncodedEvent {
    format: String,
    content: String,
}

#[derive(JsonSchema, Serialize, ToString, EnumVariantNames)]
#[serde(tag = "type", rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
enum DocumentEventType {
    Removed(DocumentRemovedEvent),
    Renamed(DocumentRenamedEvent),
    Modified(DocumentModifiedEvent),
    Encoded(DocumentEncodedEvent),
}

#[derive(JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct DocumentEvent {
    path: PathBuf,

    #[serde(flatten)]
    type_: DocumentEventType,
}

impl DocumentEvent {
    fn publish(path: PathBuf, type_: DocumentEventType) {
        let topic = match &type_ {
            DocumentEventType::Encoded(encoded) => {
                format!("encoded:{}", encoded.format)
            }
            _ => type_.to_string(),
        };
        publish(
            &format!("documents:{}:{}", path.display(), topic),
            &Self { path, type_ },
        )
    }
}

/// An in-memory representation of a document
#[derive(Debug, Clone, JsonSchema, Defaults, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Document {
    /// The absolute path of the document's file.
    pub path: PathBuf,

    /// The name of the document
    ///
    /// Usually the filename from the `path` but "Unnamed"
    /// for temporary documents.
    name: String,

    /// Whether or not the document is temporary
    temporary: bool,

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
    format: Option<String>,

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

    /// The root Stencila Schema node of the document
    #[serde(skip)]
    root: Option<CreativeWorkTypes>,
}

impl Document {
    /// Create a new empty document.
    ///
    /// # Arguments
    ///
    /// - `format`: The format of the document. Without this, it is
    ///    not possible to provide previews.
    ///
    /// This function is intended to be used by editors when creating
    /// a new document. The created document will be `temporary: true`
    /// and have a temporary file path.
    fn new(format: Option<String>) -> Document {
        Document {
            path: env::temp_dir().join(uuids::generate(uuids::Family::File)),
            name: "Unnamed".into(),
            temporary: true,
            format,
            ..Default::default()
        }
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

    /// Read the document from the file system and return its content.
    ///
    /// If the document does not have a `path` yet, then this is a no op.
    fn read(&mut self) -> Result<String> {
        let content = fs::read_to_string(&self.path)?;
        self.load(content, None)?;
        Ok(self.content.clone())
    }

    /// Dump the document's content to a string in its current, or
    // a different, format
    ///
    /// # Arguments
    ///
    /// - `format`: the format to dump the content as; if not supplied assumed to be
    ///    the document's existing format.
    pub fn dump(&self, format: Option<String>) -> Result<String> {
        let content = if let Some(format) = format {
            if format == "json" {
                serde_json::to_string(&self.root)?
            } else {
                "TODO".into()
            }
        } else {
            self.content.clone()
        };
        Ok(content)
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
    /// In the future, this will also trigger an `import()` to convert the `content`
    /// into a Stencila `CreativeWork` nodes and set the document's `root` (from which
    /// the conversions will be done).
    fn load(&mut self, content: String, format: Option<String>) -> Result<&Self> {
        // Set the content of the document
        self.content = content;
        if let Some(format) = format {
            self.format = Some(format)
        }

        // Import the content into the root of the document
        if let Some(format) = self.format.clone() {
            if format == "json" {
                self.root = serde_json::from_str(&self.content)?
            }
        }

        // Encode to each of the formats for which there are subscriptions
        for subscription in self.subscriptions.keys() {
            if let Some(format) = subscription.strip_prefix("encoded:") {
                if let Err(error) = self.encode(format) {
                    tracing::warn!("Unable to encode to format \"{}\": {}", format, error)
                }
            }
        }

        Ok(self)
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
    /// If the document does not have a `path` yet, then nothing will be written.
    fn write(&mut self, content: Option<String>, format: Option<String>) -> Result<&Self> {
        if let Some(content) = content {
            self.load(content, format)?;
        }

        let mut file = fs::File::create(&self.path)?;
        file.write_all(self.content.as_bytes())?;

        Ok(self)
    }

    /// Encode the document's content to another format
    ///
    /// In contrast to `convert()`, this does not rely on the presence of the file
    /// on the file system. It converts the current in-memory `root` node of the document
    /// to some other format and does not touch the file system.
    /// Intended mainly for in-application display of the document in alternative formats.
    fn encode(&self, format: &str) -> Result<String> {
        // Shortcut encoding to JSON
        if format == "json" {
            if let Some(node) = &self.root {
                return Ok(serde_json::to_string(node)?);
            }
        }

        // TODO: replace this use of temp files and convert with delegation of `encode` call
        let input = env::temp_dir().join(uuids::generate(uuids::Family::File));
        fs::write(input, self.content.clone())?;
        let output = env::temp_dir().join(uuids::generate(uuids::Family::File));
        self.convert(&output.display().to_string(), Some(format.into()))?;
        let content = fs::read_to_string(output)?;

        // Publish an event for this encoding
        DocumentEvent::publish(
            self.path.clone(),
            DocumentEventType::Encoded(DocumentEncodedEvent {
                format: format.into(),
                content: content.clone(),
            }),
        );

        Ok(content)
    }

    /// Convert the document's file to another format
    fn convert(&self, output: &str, format: Option<String>) -> Result<()> {
        let mut command = Command::new("encoda");
        command
            .arg("convert")
            .arg(self.path.display().to_string())
            .arg(output);

        if let Some(from) = self.format.clone() {
            command.arg(format!("--from={}", from));
        }

        if let Some(to) = format {
            command.arg(format!("--to={}", to));
        }

        let output = command.output()?;
        if !output.status.success() {
            bail!(
                "While attempting to convert: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        } else {
            Ok(())
        }
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
    fn save_as(&self, path: &str, format: Option<String>) -> Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(self.dump(format)?.as_bytes())?;
        Ok(())
    }

    /// Called when the file is removed from the file system
    ///
    /// Publishes a `Removed` event so that, for example, a document's
    /// tab can be updated to indicate it is deleted.
    fn removed(&self, path: PathBuf) {
        tracing::debug!("Document removed: {}", path.display());

        DocumentEvent::publish(path, DocumentEventType::Removed(DocumentRemovedEvent {}))
    }

    /// Called when the file is renamed
    ///
    /// Publishes a `Renamed` event so that, for example, a document's
    /// tab can be updated with the new file name.
    fn renamed(&self, from: PathBuf, to: PathBuf) {
        tracing::debug!("Document renamed: {} to {}", from.display(), to.display());

        DocumentEvent::publish(
            from,
            DocumentEventType::Renamed(DocumentRenamedEvent { to }),
        )
    }

    /// Called when the file is modified
    ///
    /// Reads the file into `content` and emits both a `Modified` and
    /// `Content` event so that the user can be asked if they
    /// want to load the new content into editor, or overwrite with
    /// existing editor content. Read only views may wish to only
    /// subscribe to `Content` events.
    ///
    /// If there are any subscribers to `PreviewUpdated` events
    /// will regenerate previews and emit those.
    fn modified(&mut self, path: PathBuf) {
        tracing::debug!("Document modified: {}", path.display());

        match self.read() {
            Ok(content) => DocumentEvent::publish(
                path,
                DocumentEventType::Modified(DocumentModifiedEvent { content }),
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
                DebouncedEvent::Remove(path) => lock().removed(path),
                DebouncedEvent::Rename(from, to) => lock().renamed(from, to),
                DebouncedEvent::Write(path) => lock().modified(path),
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
pub struct DocumentHandle {
    #[serde(flatten)]
    document: Arc<Mutex<Document>>,

    #[serde(skip)]
    watcher: Option<DocumentWatcher>,
}

impl DocumentHandle {
    /// Create a document handle from an existing file path.
    ///
    /// # Arguments
    ///
    /// - `path`: the path of the file to create the document from
    fn open(path: PathBuf, format: Option<String>, watch: bool) -> Result<DocumentHandle> {
        if path.is_dir() {
            bail!("Can not open a folder as a document; maybe try opening it as a project instead.")
        }

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
            path: path.clone(),
            name,
            temporary: false,
            ..Document::new(format)
        };
        document.read()?;

        let document: Arc<Mutex<Document>> = Arc::new(Mutex::new(document));

        let watcher = if watch {
            Some(DocumentWatcher::new(path, Arc::clone(&document)))
        } else {
            None
        };

        Ok(DocumentHandle { document, watcher })
    }
}

/// An in-memory store of documents
#[derive(Clone, Debug, Default)]
pub struct Documents {
    /// A mapping of file paths to open documents
    registry: HashMap<PathBuf, DocumentHandle>,
}

impl Documents {
    pub fn list(&self) -> Result<Vec<String>> {
        Ok(self
            .registry
            .keys()
            .map(|path| path.display().to_string())
            .collect::<Vec<String>>())
    }

    pub fn open(&mut self, path: &str, format: Option<String>) -> Result<Document> {
        let path = Path::new(path).canonicalize()?;
        let handle = match self.registry.entry(path.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => match DocumentHandle::open(path, format, true) {
                Ok(handle) => entry.insert(handle),
                Err(error) => return Err(error),
            },
        };
        let document = handle.document.lock().expect("Unable to lock document");
        Ok(document.clone())
    }

    pub fn close(&mut self, path: &str) -> Result<()> {
        let path = Path::new(path).canonicalize()?;
        self.registry.remove(&path);
        Ok(())
    }

    pub fn get(&mut self, path: &str) -> Result<MutexGuard<Document>> {
        let path = Path::new(path).canonicalize()?;
        if let Some(handle) = self.registry.get(&path) {
            if let Ok(guard) = handle.document.lock() {
                Ok(guard)
            } else {
                bail!("Unable to lock document")
            }
        } else {
            bail!("Document has not been opened yet")
        }
    }

    pub fn subscribe(&mut self, path: &str, topic: &str) -> Result<()> {
        self.get(&path)?.subscribe(topic)
    }

    pub fn unsubscribe(&mut self, path: &str, topic: &str) -> Result<()> {
        self.get(&path)?.unsubscribe(topic)
    }

    pub fn read(&mut self, path: &str) -> Result<String> {
        self.get(&path)?.read()
    }

    pub fn dump(&mut self, path: &str) -> Result<String> {
        self.get(&path)?.dump(None)
    }

    pub fn load(&mut self, path: &str, content: String) -> Result<()> {
        self.get(&path)?.load(content, None)?;
        Ok(())
    }

    pub fn write(&mut self, path: &str, content: Option<String>) -> Result<()> {
        self.get(&path)?.write(content, None)?;
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
        Convert(Convert),
        Schemas(Schemas),
    }

    impl Command {
        pub fn run(&self, documents: &mut Documents) -> display::Result {
            let Self { action } = self;
            match action {
                Action::List(action) => action.run(documents),
                Action::Open(action) => action.run(documents),
                Action::Close(action) => action.run(documents),
                Action::Show(action) => action.run(documents),
                Action::Convert(action) => action.run(documents),
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
        pub fn run(&self, documents: &mut Documents) -> display::Result {
            let Self { file } = self;
            documents.open(file, None)?;
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
        pub fn run(&self, documents: &mut Documents) -> display::Result {
            let Self { file, format } = self;
            let document = documents.open(file, format.clone())?;
            display::value(document)
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
        pub fn run(&self, documents: &mut Documents) -> display::Result {
            let Self {
                input,
                output,
                from,
                to,
            } = self;
            let document = documents.open(input, from.clone())?;
            document.convert(output, to.clone())?;
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
