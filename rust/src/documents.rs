use crate::{
    errors::attempt,
    formats::{Format, FORMATS},
    methods::{
        compile::compile,
        decode::decode,
        encode::{self, encode},
    },
    pubsub::publish,
    utils::{schemas, uuids},
};
use defaults::Defaults;
use eyre::{bail, Result};
use notify::DebouncedEvent;
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::time::{Duration, Instant};
use std::{
    collections::{hash_map::Entry, HashMap},
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use stencila_schema::Node;
use strum::ToString;
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Debug, JsonSchema, Serialize, ToString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
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

    /// The format of the document, only provided for `modified` (the format
    /// of the document) and `encoded` events (the format of the encoding).
    #[schemars(schema_with = "DocumentEvent::schema_format")]
    format: Option<Format>,
}

impl DocumentEvent {
    /// Generate the JSON Schema for the `document` property to avoid nesting
    fn schema_document(_generator: &mut SchemaGenerator) -> Schema {
        schemas::typescript("Document", true)
    }

    /// Generate the JSON Schema for the `format` property to avoid nesting
    fn schema_format(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("Format", false)
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

    /// The project directory for this document.
    ///
    /// Used to restrict file links (e.g. image paths) to within
    /// the project for both security and reproducibility reasons.
    /// For documents opened from within a project, this will be project directory.
    /// For "orphan" documents (opened by themselves) this will be the
    /// parent directory of the document. When the document is compiled,
    /// an error will be returned if a file link is outside of the root.
    project: PathBuf,

    /// Whether or not the document's file is in the temporary
    /// directory.
    temporary: bool,

    /// The synchronization status of the document.
    /// This is orthogonal to `temporary` because a document's
    /// `content` can be synced or un-synced with the file system
    /// regardless of whether or not its `path` is temporary..
    #[def = "DocumentStatus::Unread"]
    status: DocumentStatus,

    /// The last time that the document was written to disk.
    /// Used to ignore subsequent file modification events.
    #[serde(skip)]
    last_write: Option<Instant>,

    /// The name of the document
    ///
    /// Usually the filename from the `path` but "Unnamed"
    /// for temporary documents.
    name: String,

    /// The format of the document.
    ///
    /// On initialization, this is inferred, if possible, from the file name extension
    /// of the document's `path`. However, it may change whilst the document is
    /// open in memory (e.g. if the `load` function sets a different format).
    #[def = "Format::unknown(\"unknown\")"]
    #[schemars(schema_with = "Document::schema_format")]
    format: Format,

    /// Whether a HTML preview of the document is supported
    ///
    /// This is determined by the type of the `root` node of the document.
    /// Will be `true` if the `root` is a type for which HTML previews are
    /// implemented e.g. `Article`, `ImageObject` and `false` if the `root`
    /// is `None`, or of some other type e.g. `Entity`.
    ///
    /// This flag is intended for dynamically determining whether to open
    /// a preview panel for a document by default. Regardless of its value,
    /// a user should be able to open a preview panel, in HTML or some other
    /// format, for any document.
    #[def = "false"]
    previewable: bool,

    /// The current UTF8 string content of the document.
    ///
    /// When a document is `read()` from a file the `content` is the content
    /// of the file. The `content` may subsequently be changed using
    /// the `load()` function. A call to `write()` will write the content
    /// back to `path`.
    #[serde(skip)]
    content: String,

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
    /// Generate the JSON Schema for the `format` property to avoid duplicated
    /// inline type.
    fn schema_format(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("Format", true)
    }

    /// Create a new empty document.
    ///
    /// # Arguments
    ///
    /// - `format`: The format of the document; defaults to plain text.
    ///
    /// This function is intended to be used by editors when creating
    /// a new document. The created document will be `temporary: true`
    /// and have a temporary file path.
    fn new(format: Option<String>) -> Document {
        let id = uuids::generate(uuids::Family::Document);

        let path = env::temp_dir().join(uuids::generate(uuids::Family::File));
        // Ensure that the file exists
        if !path.exists() {
            fs::write(path.clone(), "").expect("Unable to write temporary file");
        }

        let project = path
            .parent()
            .expect("Unable to get path parent")
            .to_path_buf();

        let format = FORMATS.match_path(&format.unwrap_or_else(|| "txt".to_string()));

        Document {
            id,
            path,
            project,
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
    /// - `path`: The path of the file to create the document from
    ///
    /// - `format`: The format of the document. If `None` will be inferred from
    ///             the path's file extension.
    /// TODO: add project: Option<PathBuf> so that project can be explictly set
    async fn open<P: AsRef<Path>>(path: P, format: Option<String>) -> Result<Document> {
        // Canonicalize and ensure the path exists
        let path = path.as_ref().canonicalize()?;

        // Create a new document with unique id
        let mut document = Document {
            id: uuids::generate(uuids::Family::Document),
            ..Default::default()
        };

        // Apply path and format arguments
        document.alter(Some(path), format)?;

        // Attempt to read the document from the file
        attempt(document.read().await);

        Ok(document)
    }

    /// Alter properties of the document
    ///
    /// # Arguments
    ///
    /// - `path`: The path of document's file
    ///
    /// - `format`: The format of the document. If `None` will be inferred from
    ///             the path's file extension.
    pub fn alter<P: AsRef<Path>>(&mut self, path: Option<P>, format: Option<String>) -> Result<()> {
        if let Some(path) = &path {
            let path = path.as_ref();

            if path.is_dir() {
                bail!("Can not open a folder as a document; maybe try opening it as a project instead.")
            }

            self.project = path
                .parent()
                .expect("Unable to get path parent")
                .to_path_buf();

            self.name = path
                .file_name()
                .map(|os_str| os_str.to_string_lossy())
                .unwrap_or_else(|| "Untitled".into())
                .into();

            self.path = path.to_path_buf();
            self.temporary = false;
            self.status = DocumentStatus::Unwritten;
        }

        if let Some(format) = format {
            self.format = FORMATS.match_path(&format);
        } else if let Some(path) = path {
            self.format = FORMATS.match_path(&path);
        };

        self.previewable = self.format.preview;

        Ok(())
    }

    /// Read the document from the file system, update it and return its content.
    ///
    /// Sets `status` to `Synced`. For binary files, does not actually read the content
    /// but will update the document nonetheless (possibly delegating the actual read
    /// to a binary or plugin)
    async fn read(&mut self) -> Result<String> {
        let content = if !self.format.binary {
            let content = fs::read_to_string(&self.path)?;
            self.load(content.clone(), None).await?;
            content
        } else {
            self.update().await?;
            "".to_string()
        };
        self.status = DocumentStatus::Synced;
        Ok(content)
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
        self.last_write = Some(Instant::now());

        Ok(())
    }

    /// Write the document to the file system, as an another file, possibly in
    /// another format.
    ///
    /// # Arguments
    ///
    /// - `path`: the path for the new file.
    /// - `format`: the format to dump the content as; if not supplied assumed to be
    ///    the document's existing format.
    /// - `theme`: theme to apply to the new document (HTML and PDF only).
    ///
    /// Note: this does not change the `path`, `format` or `status` of the current
    /// document.
    async fn write_as<P: AsRef<Path>>(
        &self,
        path: P,
        format: Option<String>,
        theme: Option<String>,
    ) -> Result<()> {
        let path = path.as_ref();
        let format = format.unwrap_or_else(|| {
            path.extension().map_or_else(
                || self.format.name.clone(),
                |ext| ext.to_string_lossy().to_string(),
            )
        });
        let output = ["file://", &path.display().to_string()].concat();

        let content = if let Some(root) = &self.root {
            let mut options = encode::Options::default();
            if let Some(theme) = theme {
                options.theme = theme
            }
            encode(root, &output, &format, Some(options)).await?
        } else {
            tracing::warn!("Document has no root node");
            "".to_string()
        };

        if !content.starts_with("file://") {
            fs::write(path, content)?;
        }

        Ok(())
    }

    /// Dump the document's content to a string in its current, or
    /// alternative, format.
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
            encode(root, "string://", &format, None).await
        } else {
            tracing::warn!("Document has no root node");
            Ok(String::new())
        }
    }

    /// Load content into the document
    ///
    /// # Arguments
    ///
    /// - `content`: the content to load into the document
    /// - `format`: the format of the content; if not supplied assumed to be
    ///    the document's existing format.
    async fn load(&mut self, content: String, format: Option<String>) -> Result<()> {
        // Set the `content` and `status` of the document
        self.content = content;
        self.status = DocumentStatus::Unwritten;
        if let Some(format) = format {
            self.format = FORMATS.match_path(&format)
        }

        self.update().await
    }

    /// Update the `root` (and associated properties) of the document and publish updated encodings
    ///
    /// Publishes `encoded:` events for each of the formats subscribed to.
    /// Error results from this function (e.g. compile errors)
    /// should generally not be bubbled up.
    async fn update(&mut self) -> Result<()> {
        tracing::debug!(
            "Updating document '{}' at '{}'",
            self.id,
            self.path.display()
        );

        // Decode the content into the `root` node of the document
        let format = &self.format.name;
        let mut root = if self.format.binary {
            let path = self.path.display().to_string();
            let input = ["file://", &path].concat();
            decode(&input, format).await?
        } else {
            decode(&self.content, format).await?
        };

        #[cfg(feature = "reshape")]
        {
            // Reshape the `root` according to preferences
            use crate::methods::reshape::{self, reshape};
            reshape(&mut root, reshape::Options::default())?;
        }

        // Compile the `root` and update document dependencies
        let _compilation = compile(&mut root, &self.path, &self.project)?;

        // Encode the `root` into each of the formats for which there are subscriptions
        for subscription in self.subscriptions.keys() {
            if let Some(format) = subscription.strip_prefix("encoded:") {
                tracing::debug!("Encoding document '{}' to '{}'", self.id, format);
                match encode(&root, "string://", format, None).await {
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

        // Determine if the document is preview-able, based on the type of the root
        // This list of types should be updated as HTML encoding is implemented for each.
        self.previewable = matches!(
            root,
            Node::Article(..)
                | Node::ImageObject(..)
                | Node::AudioObject(..)
                | Node::VideoObject(..)
        );

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
    #[allow(unreachable_code)]
    fn query(&self, query: &str, lang: &str) -> Result<serde_json::Value> {
        let result = match lang {
            #[cfg(feature = "query-jmespath")]
            "jmespath" => {
                let expr = jmespatch::compile(query)?;
                let result = expr.search(&self.root)?;
                serde_json::to_value(result)?
            }
            #[cfg(feature = "query-jsonptr")]
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
        let format = format.map(|name| FORMATS.match_name(&name));

        let topic = match type_ {
            DocumentEventType::Encoded => format!(
                "encoded:{}",
                format
                    .clone()
                    .map_or_else(|| "undef".to_string(), |format| format.name)
            ),
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
    #[allow(dead_code)]
    fn renamed(&mut self, from: PathBuf, to: PathBuf) {
        tracing::debug!("Document renamed: {} to {}", from.display(), to.display());

        // If the document has been moved out of its project then we need to reassign `project`
        // (to ensure that files in the old project can not be linked to).
        if to.strip_prefix(&self.project).is_err() {
            self.project = match to.parent() {
                Some(path) => path.to_path_buf(),
                None => to.clone(),
            }
        }

        self.path = to;

        self.publish(DocumentEventType::Renamed, None, None)
    }

    const LAST_WRITE_MUTE_MILLIS: u64 = 300;

    /// Called when the file is modified
    ///
    /// Reads the file into `content` and emits a `Modified` event so that the user
    /// can be asked if they want to load the new content into editor, or overwrite with
    /// existing editor content.
    ///
    /// Will ignore any events within a small duration of `write()` being called.
    async fn modified(&mut self, path: PathBuf) {
        if let Some(last_write) = self.last_write {
            if last_write.elapsed() < Duration::from_millis(Document::LAST_WRITE_MUTE_MILLIS) {
                return;
            }
        }

        tracing::debug!("Document modified: {}", path.display());

        self.status = DocumentStatus::Unread;

        match self.read().await {
            Ok(content) => self.publish(
                DocumentEventType::Modified,
                Some(content),
                Some(self.format.name.clone()),
            ),
            Err(error) => tracing::error!("While attempting to read modified file: {}", error),
        }
    }
}

#[derive(Debug)]
pub struct DocumentHandler {
    /// The document being handled.
    document: Arc<Mutex<Document>>,

    /// The watcher thread's channel sender.
    ///
    /// Held so that when this handler is dropped, the
    /// watcher thread is ended.
    watcher: Option<crossbeam_channel::Sender<()>>,

    /// The event handler thread's join handle.
    ///
    /// Held so that when this handler is dropped, the
    /// event handler thread is aborted.
    handler: Option<JoinHandle<()>>,
}

impl Clone for DocumentHandler {
    fn clone(&self) -> Self {
        DocumentHandler {
            document: self.document.clone(),
            watcher: None,
            handler: None,
        }
    }
}

impl Drop for DocumentHandler {
    fn drop(&mut self) {
        match &self.handler {
            Some(handler) => handler.abort(),
            None => {}
        }
    }
}

impl DocumentHandler {
    /// Create a new document handler.
    ///
    /// # Arguments
    ///
    /// - `document`: The document that this handler is for.
    /// - `watch`: Whether to watch the document (e.g. not for temporary, new files)
    fn new(document: Document, watch: bool) -> DocumentHandler {
        let id = document.id.clone();
        let path = document.path.clone();

        let document = Arc::new(Mutex::new(document));
        let (watcher, handler) = if watch {
            let (watcher, handler) = DocumentHandler::watch(id, path, Arc::clone(&document));
            (Some(watcher), Some(handler))
        } else {
            (None, None)
        };

        DocumentHandler {
            document,
            watcher,
            handler,
        }
    }

    const WATCHER_DELAY_MILLIS: u64 = 100;

    /// Watch the document.
    ///
    /// It is necessary to have a file watcher that is separate from a project directory watcher
    /// for documents that are opened independent of a project (a.k.a. orphan documents).
    ///
    /// It is also necessary for this watcher to be on the parent folder of the document
    /// (which, for some documents may be concurrent with the watcher for the project) and to filter
    /// events related to the file. That is necessary because some events are otherwise
    /// not captured e.g. file renames (delete and then create) and file writes by some software
    /// (e.g. LibreOffice deletes and then creates a file instead of just writing it).
    fn watch(
        id: String,
        path: PathBuf,
        document: Arc<Mutex<Document>>,
    ) -> (crossbeam_channel::Sender<()>, JoinHandle<()>) {
        let (thread_sender, thread_receiver) = crossbeam_channel::bounded(1);
        let (async_sender, mut async_receiver) = tokio::sync::mpsc::channel(100);

        let path_cloned = path.clone();

        // Standard thread to run blocking sync file watcher
        std::thread::spawn(move || -> Result<()> {
            use notify::{watcher, RecursiveMode, Watcher};

            let (watcher_sender, watcher_receiver) = std::sync::mpsc::channel();
            let mut watcher = watcher(
                watcher_sender,
                Duration::from_millis(DocumentHandler::WATCHER_DELAY_MILLIS),
            )?;
            let parent = path.parent().unwrap_or(&path);
            watcher.watch(&parent, RecursiveMode::NonRecursive)?;

            // Event checking timeout. Can be quite long since only want to check
            // whether we can end this thread.
            let timeout = Duration::from_millis(100);

            let path_string = path.display().to_string();
            let span = tracing::info_span!("document_watch", path = path_string.as_str());
            let _enter = span.enter();
            tracing::debug!(
                "Starting document watcher for '{}' at '{}'",
                id,
                path_string
            );
            loop {
                // Check for an event. Use `recv_timeout` so we don't
                // get stuck here and will do following check that ends this
                // thread if the owning `DocumentHandler` is dropped
                if let Ok(event) = watcher_receiver.recv_timeout(timeout) {
                    tracing::debug!(
                        "Event for document '{}' at '{}': {:?}",
                        id,
                        path_string,
                        event
                    );
                    if async_sender.blocking_send(event).is_err() {
                        break;
                    }
                }
                // Check to see if this thread should be ended
                if let Err(crossbeam_channel::TryRecvError::Disconnected) =
                    thread_receiver.try_recv()
                {
                    break;
                }
            }
            tracing::debug!("Ending document watcher for '{}' at '{}'", id, path_string);

            // Drop the sync send so that the event handling thread also ends
            drop(async_sender);

            Ok(())
        });

        // Async task to handle events
        let handler = tokio::spawn(async move {
            let mut document_path = path_cloned;
            tracing::debug!("Starting document handler");
            while let Some(event) = async_receiver.recv().await {
                match event {
                    DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                        if path == document_path {
                            document.lock().await.modified(path).await
                        }
                    }
                    DebouncedEvent::Remove(path) => {
                        if path == document_path {
                            document.lock().await.deleted(path)
                        }
                    }
                    DebouncedEvent::Rename(from, to) => {
                        if from == document_path {
                            document_path = to.clone();
                            document.lock().await.renamed(from, to)
                        }
                    }
                    _ => {}
                }
            }
            // Because we abort this thread, this entry may never get
            // printed (only if the `async_sender` is dropped before this is aborted)
            tracing::debug!("Ending document handler");
        });

        (thread_sender, handler)
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

    /// List documents that are currently open
    ///
    /// Returns a vector of document paths (relative to the current working directory)
    pub async fn list(&self) -> Result<Vec<String>> {
        let cwd = std::env::current_dir()?;
        let mut paths = Vec::new();
        for document in self.registry.values() {
            let path = &document.document.lock().await.path;
            let path = match pathdiff::diff_paths(path, &cwd) {
                Some(path) => path,
                None => path.clone(),
            };
            let path = path.display().to_string();
            paths.push(path);
        }
        Ok(paths)
    }

    /// Create a new empty document
    pub fn create(&mut self, format: Option<String>) -> Result<Document> {
        let document = Document::new(format);
        let handler = DocumentHandler::new(document.clone(), false);
        self.registry.insert(document.id.clone(), handler);
        Ok(document)
    }

    /// Open a document
    ///
    /// # Arguments
    ///
    /// - `path`: The path of the document to open
    /// - `format`: The format to open the document as (inferred from filename extension if not supplied)
    ///
    /// If the document has already been opened, it will not be re-opened, but rather the existing
    /// in-memory instance will be returned.
    pub async fn open<P: AsRef<Path>>(
        &mut self,
        path: P,
        format: Option<String>,
    ) -> Result<Document> {
        let path = Path::new(path.as_ref()).canonicalize()?;

        for handler in self.registry.values() {
            let document = handler.document.lock().await;
            if document.path == path {
                return Ok(document.clone());
            }
        }

        let document = Document::open(path, format).await?;
        let handler = DocumentHandler::new(document.clone(), true);
        self.registry.insert(document.id.clone(), handler);
        Ok(document)
    }

    /// Close a document
    ///
    /// # Arguments
    ///
    /// - `id_or_path`: The id or path of the document to close
    ///
    /// If `id_or_path` matches an existing document `id` then that document will
    /// be closed. Otherwise a search will be done and the first document with a matching
    /// path will be closed.
    pub async fn close<P: AsRef<Path>>(&mut self, id_or_path: P) -> Result<()> {
        let id_or_path_path = id_or_path.as_ref();
        let id_or_path_string = id_or_path_path.to_string_lossy().to_string();
        let mut id_to_remove = String::new();

        if self.registry.contains_key(&id_or_path_string) {
            id_to_remove = id_or_path_string
        } else {
            let path = id_or_path_path.canonicalize()?;
            for handler in self.registry.values() {
                let document = handler.document.lock().await;
                if document.path == path {
                    id_to_remove = document.id.clone();
                    break;
                }
            }
        };
        self.registry.remove(&id_to_remove);

        Ok(())
    }

    pub fn get(&mut self, id: &str) -> Result<Arc<Mutex<Document>>> {
        if let Some(handler) = self.registry.get(id) {
            Ok(handler.document.clone())
        } else {
            bail!("No document with id {}", id)
        }
    }

    pub async fn read(&mut self, id: &str) -> Result<String> {
        self.get(&id)?.lock().await.read().await
    }

    pub async fn write(&mut self, id: &str, content: Option<String>) -> Result<()> {
        self.get(&id)?.lock().await.write(content, None).await
    }

    pub async fn write_as(
        &mut self,
        id: &str,
        path: &str,
        format: Option<String>,
        theme: Option<String>,
    ) -> Result<()> {
        self.get(&id)?
            .lock()
            .await
            .write_as(path, format, theme)
            .await
    }

    pub async fn dump(&mut self, id: &str, format: Option<String>) -> Result<String> {
        self.get(&id)?.lock().await.dump(format).await
    }

    pub async fn load(&mut self, id: &str, content: String) -> Result<()> {
        self.get(&id)?.lock().await.load(content, None).await
    }

    pub async fn subscribe(&mut self, id: &str, topic: &str) -> Result<()> {
        self.get(&id)?.lock().await.subscribe(topic)
    }

    pub async fn unsubscribe(&mut self, id: &str, topic: &str) -> Result<()> {
        self.get(&id)?.lock().await.unsubscribe(topic)
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
        pub async fn run(self, documents: &mut Documents) -> display::Result {
            let Self { action } = self;
            match action {
                Action::List(action) => action.run(documents).await,
                Action::Open(action) => action.run(documents).await,
                Action::Close(action) => action.run(documents).await,
                Action::Show(action) => action.run(documents).await,
                Action::Query(action) => action.run(documents).await,
                Action::Convert(action) => action.run(documents).await,
                Action::Schemas(action) => action.run(),
            }
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "List open documents",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}

    impl List {
        pub async fn run(&self, documents: &mut Documents) -> display::Result {
            let list = documents.list().await?;
            display::value(list)
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Open a document",
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
        about = "Close a document",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Close {
        /// The path of the document file
        #[structopt(default_value = ".")]
        pub file: String,
    }

    impl Close {
        pub async fn run(&self, documents: &mut Documents) -> display::Result {
            let Self { file } = self;
            documents.close(file).await?;
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
        pub input: PathBuf,

        /// The path of the output document
        pub output: PathBuf,

        /// The format of the input (defaults to being inferred from the file extension or content type)
        #[structopt(short, long)]
        from: Option<String>,

        /// The format of the output (defaults to being inferred from the file extension)
        #[structopt(short, long)]
        to: Option<String>,

        /// The theme to apply to the output (only for HTML and PDF)
        #[structopt(short = "e", long)]
        theme: Option<String>,
    }

    impl Convert {
        pub async fn run(self, _documents: &mut Documents) -> display::Result {
            let Self {
                input,
                output,
                from,
                to,
                theme,
            } = self;
            let document = Document::open(input, from).await?;
            document.write_as(output, to, theme).await?;
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
        assert_eq!(doc.format.name, "txt");
        assert_eq!(doc.content, "");
        assert!(doc.root.is_none());
        assert_eq!(doc.subscriptions, hashmap! {});

        let doc = Document::new(Some("md".to_string()));
        assert!(doc.path.starts_with(env::temp_dir()));
        assert!(doc.temporary);
        assert!(matches!(doc.status, DocumentStatus::Synced));
        assert_eq!(doc.format.name, "md");
        assert_eq!(doc.content, "");
        assert!(doc.root.is_none());
        assert_eq!(doc.subscriptions, hashmap! {});
    }

    #[tokio::test]
    async fn document_open() -> Result<()> {
        let fixtures = &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("fixtures")
            .join("articles")
            .canonicalize()?;

        for file in vec!["elife-small.json", "era-plotly.json"] {
            let doc = Document::open(fixtures.join(file), None).await?;
            assert!(doc.path.starts_with(fixtures));
            assert!(!doc.temporary);
            assert!(matches!(doc.status, DocumentStatus::Synced));
            assert_eq!(doc.format.name, "json");
            assert!(doc.content.len() > 0);
            assert!(doc.root.is_some());
            assert_eq!(doc.subscriptions, hashmap! {});
        }

        Ok(())
    }
}
