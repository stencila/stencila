use crate::utils::schemas;
use defaults::Defaults;
use events::publish;
use eyre::{bail, Result};
use formats::FormatSpec;
use graph::Graph;
use graph_triples::Relations;
use kernels::KernelSpace;
use maplit::hashset;
use node_address::AddressMap;
use node_execute::{compile, Planner};
use node_patch::{apply, diff, merge, Patch};
use node_pointer::{resolve, Pointer};
use node_reshape::reshape;
use notify::DebouncedEvent;
use once_cell::sync::Lazy;
use path_slash::PathBufExt;
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    env, fs,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use stencila_schema::{Article, Node};
use strum::Display;
use tokio::{
    sync::{mpsc, Mutex, RwLock},
    task::JoinHandle,
};

#[derive(Debug, JsonSchema, Serialize, Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
enum DocumentEventType {
    Deleted,
    Renamed,
    Modified,
    Patched,
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
    format: Option<FormatSpec>,

    /// The `Patch` associated with a `Patched` event
    #[schemars(schema_with = "DocumentEvent::schema_patch")]
    patch: Option<Patch>,
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

    /// Generate the JSON Schema for the `patch` property to avoid nesting
    fn schema_patch(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("Patch", false)
    }
}

/// The status of a document with respect to on-disk synchronization
#[derive(Debug, Clone, JsonSchema, Serialize, Display)]
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
#[derive(Debug, Defaults, JsonSchema, Serialize)]
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
    /// Usually the filename from the `path` but "Untitled"
    /// for temporary documents.
    name: String,

    /// The format of the document.
    ///
    /// On initialization, this is inferred, if possible, from the file name extension
    /// of the document's `path`. However, it may change whilst the document is
    /// open in memory (e.g. if the `load` function sets a different format).
    #[def = "FormatSpec::unknown(\"unknown\")"]
    #[schemars(schema_with = "Document::schema_format")]
    format: FormatSpec,

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
    ///
    /// Skipped during serialization because will often be large.
    #[serde(skip)]
    content: String,

    /// The root Stencila Schema node of the document
    ///
    /// Can be any type of `Node` but defaults to an empty `Article`.
    ///
    /// A [`RwLock`] to enable separate, concurrent tasks to read (e.g. for dumping to some
    /// format) and write (e.g. to apply patches from clients) the node.
    ///
    /// Skipped during serialization because will often be large.
    #[def = "RwLock::new(Node::Article(Article::default()))"]
    #[serde(skip)]
    pub root: RwLock<Node>,

    /// Addresses of nodes in `root` that have an `id`
    ///
    /// Used to fetch a particular node (and do something with it like `patch`
    /// or `execute` it) rather than walking the node tree looking for it.
    /// It is necessary to use [`Address`] here (rather than say raw pointers) because
    /// pointers or references will change as the document is patched.
    /// These addresses are shifted when the document is patched to account for this.
    #[schemars(schema_with = "Document::schema_addresses")]
    addresses: AddressMap,

    /// The execution planner for this document.
    #[schemars(skip)]
    planner: Planner,

    /// The kernel space for this document.
    ///
    /// This is where document variables are stored and executable nodes such as
    /// `CodeChunk`s and `Parameters`s are executed.
    #[serde(skip)]
    kernels: Arc<KernelSpace>,

    /// The set of dependency relations between this document, or nodes in this document,
    /// and other resources.
    ///
    /// Relations may be external (e.g. the document links to another `Resource::File`),
    /// or internal (e.g. the second code chunk uses a `Resource::Symbol` defined in the
    /// first code chunk).
    ///
    /// Stored for use in building the project's graph, but that may be removed
    /// in the future. Not serialized since this information is in `self.graph`.
    #[serde(skip)]
    pub relations: Relations,

    /// The document's dependency graph
    ///
    /// This is derived from `relations`.
    #[serde(skip_deserializing)]
    #[schemars(skip)]
    pub graph: Graph,

    /// The clients that are subscribed to each topic for this document
    ///
    /// Keeping track of client ids per topics allows for a some
    /// optimizations. For example, events will only be published on topics that have at least one
    /// subscriber.
    ///
    /// Valid subscription topics are the names of the `DocumentEvent` types:
    ///
    /// - `removed`: published when document file is deleted
    /// - `renamed`: published when document file is renamed
    /// - `modified`: published when document file is modified
    /// - `encoded:<format>` published when a document's content
    ///    is changed internally or externally and  conversions have been
    ///    completed e.g. `encoded:html`
    subscriptions: HashMap<String, HashSet<String>>,
}

#[allow(unused)]
impl Document {
    /// Generate the JSON Schema for the `format` property to avoid duplicated
    /// inline type.
    fn schema_format(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("Format", true)
    }

    /// Generate the JSON Schema for the `addresses` property to avoid duplicated types.
    fn schema_addresses(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("Record<string, Address>", true)
    }

    /// Create a new empty document.
    ///
    /// # Arguments
    ///
    /// - `path`: The path of the document; defaults to a temporary path.
    /// - `format`: The format of the document; defaults to plain text.
    ///
    /// This function is intended to be used by editors when creating
    /// a new document. If the `path` is not specified, the created document
    /// will be `temporary: true` and have a temporary file path.
    fn new(path: Option<PathBuf>, format: Option<String>) -> Document {
        let id = uuids::generate("do").to_string();

        let format = if let Some(format) = format {
            formats::match_path(&format)
        } else if let Some(path) = path.as_ref() {
            formats::match_path(path)
        } else {
            formats::match_name("txt")
        }
        .spec();
        let previewable = format.preview;

        let (path, name, temporary) = match path {
            Some(path) => {
                let name = path
                    .file_name()
                    .map(|os_str| os_str.to_string_lossy())
                    .unwrap_or_else(|| "Untitled".into())
                    .into();

                (path, name, false)
            }
            None => {
                let path = env::temp_dir().join(
                    [
                        uuids::generate("fi").to_string(),
                        ".".to_string(),
                        format.extension.clone(),
                    ]
                    .concat(),
                );
                // Ensure that the file exists
                if !path.exists() {
                    fs::write(path.clone(), "").expect("Unable to write temporary file");
                }

                let name = "Untitled".into();

                (path, name, true)
            }
        };

        let project = path
            .parent()
            .expect("Unable to get path parent")
            .to_path_buf();

        Document {
            id,
            path,
            project,
            temporary,
            status: DocumentStatus::Synced,
            name,
            format,
            previewable,
            ..Default::default()
        }
    }

    /// Create a representation of the document
    ///
    /// Used to represent the document in events and as the return value of functions without
    /// to provide properties such as `path` and `status` without cloning things such as
    /// its `kernels`.
    pub fn repr(&self) -> Self {
        Self {
            id: self.id.clone(),
            path: self.path.clone(),
            project: self.project.clone(),
            temporary: self.temporary,
            status: self.status.clone(),
            name: self.name.clone(),
            format: self.format.clone(),
            previewable: self.previewable,
            addresses: self.addresses.clone(),
            planner: self.planner.clone(),
            graph: self.graph.clone(),
            subscriptions: self.subscriptions.clone(),
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
    pub async fn open<P: AsRef<Path>>(path: P, format: Option<String>) -> Result<Document> {
        let path = path.as_ref();

        // Create a new document with unique id
        let mut document = Document {
            id: uuids::generate("do").to_string(),
            ..Default::default()
        };

        // Apply path and format arguments
        document.alter(Some(path), format).await?;

        // Attempt to read the document from the file
        match document.read(true).await {
            Ok(..) => (),
            Err(error) => tracing::warn!("While reading document `{}`: {}", path.display(), error),
        };

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
    pub async fn alter<P: AsRef<Path>>(
        &mut self,
        path: Option<P>,
        format: Option<String>,
    ) -> Result<()> {
        if let Some(path) = &path {
            let path = path.as_ref().canonicalize()?;

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

            self.path = path;
            self.temporary = false;
            self.status = DocumentStatus::Unwritten;
        }

        if let Some(format) = format {
            self.format = formats::match_path(&format).spec();
        } else if let Some(path) = path {
            self.format = formats::match_path(&path).spec();
        };

        self.previewable = self.format.preview;

        // Given that the `format` may have changed, it is necessary
        // to update the `root` of the document
        self.update(true).await?;

        Ok(())
    }

    /// Read the document from the file system, update it and return its content.
    ///
    /// # Arguments
    ///
    /// - `force_load`: if `false` then if the file is empty, or is the same as the existing
    ///                 content then do not load the content into the document
    ///
    /// Using `force_load: false` is recommended when calling this function in response to
    /// file modification events as writes in quick succession can cause the file to be momentarily
    /// empty when read.
    ///
    /// Sets `status` to `Synced`. For binary files, does not actually read the content
    /// but will update the document nonetheless (possibly delegating the actual read
    /// to a binary or plugin)
    pub async fn read(&mut self, force_load: bool) -> Result<String> {
        let content = if !self.format.binary {
            let content = fs::read_to_string(&self.path)?;
            if force_load || (!content.is_empty() && content != self.content) {
                self.load(content.clone(), None).await?;
            }
            content
        } else {
            self.update(true).await?;
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
    pub async fn write(&mut self, content: Option<String>, format: Option<String>) -> Result<()> {
        if let Some(content) = content {
            self.load(content, format.clone()).await?;
        }

        let content_to_write = if let Some(input_format) = format.as_ref() {
            let input_format = formats::match_path(&input_format).spec();
            if input_format != self.format {
                self.dump(None).await?
            } else {
                self.content.clone()
            }
        } else {
            self.content.clone()
        };

        fs::write(&self.path, content_to_write.as_bytes())?;
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
    pub async fn write_as<P: AsRef<Path>>(
        &self,
        path: P,
        format: Option<String>,
        theme: Option<String>,
    ) -> Result<()> {
        let path = path.as_ref();

        let format = format.unwrap_or_else(|| {
            path.extension().map_or_else(
                || self.format.extension.clone(),
                |ext| ext.to_string_lossy().to_string(),
            )
        });

        let mut options = codecs::EncodeOptions {
            standalone: true,
            ..Default::default()
        };
        if let Some(theme) = theme {
            options.theme = theme
        }

        let root = &*self.root.read().await;
        codecs::to_path(root, path, &format, Some(options)).await?;

        Ok(())
    }

    /// Dump the document's content to a string in its current, or
    /// alternative, format.
    ///
    /// # Arguments
    ///
    /// - `format`: the format to dump the content as; if not supplied assumed to be
    ///    the document's existing format.
    pub async fn dump(&self, format: Option<String>) -> Result<String> {
        let format = match format {
            Some(format) => format,
            None => return Ok(self.content.clone()),
        };

        let root = &*self.root.read().await;
        codecs::to_string(root, &format, None).await
    }

    /// Load content into the document
    ///
    /// If the format of the new content is different to the document's format
    /// then the content will be converted to the document's format.
    ///
    /// # Arguments
    ///
    /// - `content`: the content to load into the document
    /// - `format`: the format of the content; if not supplied assumed to be
    ///    the document's existing format.
    pub async fn load(&mut self, content: String, format: Option<String>) -> Result<()> {
        let mut decode_content = true;
        if let Some(format) = format {
            let other_format = formats::match_path(&format).spec();
            if other_format != self.format {
                let node = codecs::from_str(&content, &other_format.extension, None).await?;
                if !self.format.binary {
                    self.content = codecs::to_string(&node, &self.format.extension, None).await?;
                }
                let mut root = &mut *self.root.write().await;
                *root = node;
                decode_content = false;
            } else {
                self.content = content;
            }
        } else {
            self.content = content;
        };
        self.status = DocumentStatus::Unwritten;

        self.update(decode_content).await
    }

    /// Generate a [`Patch`] describing the operations needed to modify this
    /// document so that it is equal to another.
    pub async fn diff(&self, other: &Document) -> Result<Patch> {
        let me = &*self.root.read().await;
        let other = &*other.root.read().await;
        let patch = diff(me, other);
        Ok(patch)
    }

    /// Merge changes from two or more derived version into this document.
    ///
    /// See documentation on the [`merge`] function for how any conflicts
    /// are resolved.
    pub async fn merge(&mut self, deriveds: &[Document]) -> Result<()> {
        let mut guard = self.root.write().await;

        // Need to store `let` bindings to read guards before dereferencing them
        let mut guards = Vec::new();
        for derived in deriveds {
            let guard = derived.root.read().await;
            guards.push(guard)
        }
        let others: Vec<&Node> = guards.iter().map(|guard| guard.deref()).collect();

        // Do the merge into root
        merge(&mut *guard, &others);

        // TODO updating of *content from root* and publishing of events etc needs to be sorted out
        if !self.format.binary {
            self.content = codecs::to_string(&*guard, &self.format.extension, None).await?;
        }

        // Drop root guard to allow update
        drop(guard);

        self.update(false).await?;

        Ok(())
    }

    /// Resolve a node within the current document using an id
    ///
    /// This does not use `&mut self` to avoid the "cannot borrow as mutable more than once at a time"
    /// error in other methods where it is used.
    pub fn resolve<'root>(
        root: &'root mut Option<Node>,
        address_map: &AddressMap,
        node_id: Option<String>,
    ) -> Result<Pointer<'root>> {
        let root = match root {
            Some(root) => root,
            None => bail!("Document does not have a `root` node from which to resolve"),
        };

        let node_id = if let Some(node_id) = node_id {
            node_id
        } else {
            return Ok(Pointer::Node(root));
        };

        let address = if let Some(address) = address_map.get(&node_id) {
            if address.is_empty() {
                return Ok(Pointer::Node(root));
            }
            Some(address.clone())
        } else {
            tracing::warn!(
                "Unregistered node id `{}`; will attempt to `find()` node",
                node_id
            );
            None
        };

        resolve(root, address, Some(node_id))
    }

    /// Apply a [`Patch`] to this document
    ///
    /// # Arguments
    ///
    /// - `patch`: the patch to apply
    #[tracing::instrument(skip(self, patch))]
    pub async fn patch(&mut self, mut patch: Patch) -> Result<()> {
        tracing::debug!("Patching document `{}`", self.id);

        let topic = self.topic("patched");
        let subscribers = self.subscribers("patched");

        let root = &mut *self.root.write().await;

        // If the patch has a `target` but no `address` then use `address_map` to populate the address
        // of faster patch application
        if let (None, Some(node_id)) = (&patch.address, &patch.target) {
            if let Some(address) = self.addresses.get(node_id) {
                patch.address = Some(address.clone());
            }
        }

        // Apply the patch and update self
        apply(root, &patch);

        // Publish the patch for other subscribers
        if !patch.is_empty() && subscribers > 0 {
            patch.prepublish(root);
            publish(
                &topic,
                &DocumentEvent {
                    type_: DocumentEventType::Patched,
                    document: Document::new(None, None),
                    content: None,
                    format: None,
                    patch: Some(patch),
                },
            );
        }

        Ok(())
    }

    /// Execute the document, or a node within it, and publishing a patch of changes if there are any subscribers.
    #[tracing::instrument(skip(self))]
    pub async fn execute(&mut self, start: Option<String>) -> Result<()> {
        tracing::debug!("Executing document `{}`", self.id);

        let topic = self.topic("patched");

        let root = &mut *self.root.write().await;

        // Compile the `root` to update `addresses` and `planner` needed below
        let (address_map, relations, parse_map) = compile(root, &self.path, &self.project)?;
        self.addresses = address_map;
        self.planner
            .update(&self.path, &relations, parse_map, None)
            .await?;

        // Need to translate the `start` node id into a resource id to generate plan
        let start = start.map(|node_id| {
            [
                "code://",
                self.path.to_slash_lossy().as_str(),
                "#",
                &node_id,
            ]
            .concat()
        });

        let root_clone = root.clone();
        let (sender, mut receiver) = mpsc::channel::<Patch>(10);
        let patch_publisher = tokio::spawn(async move {
            while let Some(mut patch) = receiver.recv().await {
                patch.prepublish(&root_clone);
                publish(
                    &topic,
                    &DocumentEvent {
                        type_: DocumentEventType::Patched,
                        document: Document::new(None, None),
                        content: None,
                        format: None,
                        patch: Some(patch),
                    },
                )
            }
            tracing::debug!("Finished publishing patches");
        });

        let kernels = self.kernels.clone();
        self.planner
            .execute(root, &self.addresses, kernels, Some(sender), start, None)
            .await?;

        // Wait for all the patches to be sent. `patch_publisher` should finish
        // once all sender clones have been dropped.
        patch_publisher.await?;

        Ok(())
    }

    /// Update the `root` (and associated properties) of the document and publish updated encodings
    ///
    /// Publishes `encoded:` events for each of the formats subscribed to.
    /// Error results from this function (e.g. compile errors)
    /// should generally not be bubbled up.
    ///
    /// # Arguments
    ///
    /// - `decode_content`: Should the current content of the be decoded?. This
    ///                     is an optimization for situations where the `root` has
    ///                     just been decoded from the current `content`.
    async fn update(&mut self, decode_content: bool) -> Result<()> {
        tracing::debug!(
            "Updating document `{}` at `{}`",
            self.id,
            self.path.display()
        );

        // Decode the binary file or, in-memory content into the `root` node
        // of the document
        let format = &self.format.extension;
        let mut root = if self.format.binary {
            if self.path.exists() {
                tracing::debug!("Decoding document root from path");
                codecs::from_path(&self.path, format, None).await?
            } else {
                self.root.read().await.clone()
            }
        } else if !self.content.is_empty() {
            if decode_content {
                tracing::debug!("Decoding document root from content");
                codecs::from_str(&self.content, format, None).await?
            } else {
                self.root.read().await.clone()
            }
        } else {
            tracing::debug!("Setting document root to  empty article");
            Node::Article(Article::default())
        };

        // Reshape the `root`
        // TODO: Pass user options for reshaping through
        reshape(&mut root, None)?;

        // Attempt to compile the `root` to assign ids (needed for HTML rendering and patching)
        // and update intra- and inter- dependencies. No need to update `planner` now.
        match compile(&mut root, &self.path, &self.project) {
            Ok((addresses, relations, parse_map)) => {
                self.addresses = addresses;
                self.graph = Graph::from_relations(&self.path, &relations);
            }
            Err(error) => tracing::warn!(
                "While compiling document `{}`: {}",
                self.path.display(),
                error
            ),
        }

        // Publish any events for which there are subscriptions
        for subscription in self.subscriptions.keys() {
            // Generate a diff if there are any `patched` subscriptions
            if subscription == "patched" {
                tracing::debug!("Generating patch for document `{}`", self.id);
                let current_root = &*self.root.read().await;
                let mut patch = diff(current_root, &root);
                patch.prepublish(&root);
                publish(
                    &["documents:", &self.id, ":patched"].concat(),
                    &DocumentEvent {
                        type_: DocumentEventType::Patched,
                        document: self.repr(),
                        content: None,
                        format: None,
                        patch: Some(patch),
                    },
                )
            }
            // Encode the `root` into each of the formats for which there are subscriptions
            else if let Some(format) = subscription.strip_prefix("encoded:") {
                tracing::debug!("Encoding document `{}` to format `{}`", self.id, format);
                match codecs::to_string(&root, format, None).await {
                    Ok(content) => {
                        self.publish(
                            DocumentEventType::Encoded,
                            Some(content),
                            Some(format.into()),
                        );
                    }
                    Err(error) => {
                        tracing::warn!("Unable to encode to format `{}`: {}", format, error)
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
        let self_root = &mut *self.root.write().await;
        *self_root = root;

        Ok(())
    }

    /// Query the document
    ///
    /// Returns a JSON value. Returns `null` if the query does not select anything.
    pub async fn query(&self, query: &str, lang: &str) -> Result<serde_json::Value> {
        let root = &*self.root.read().await;
        let result = match lang {
            #[cfg(feature = "query-jmespath")]
            "jmespath" => {
                let expr = jmespatch::compile(query)?;
                let result = expr.search(root)?;
                serde_json::to_value(result)?
            }
            #[cfg(feature = "query-jsonptr")]
            "jsonptr" => {
                let data = serde_json::to_value(root)?;
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

    /// Generate a topic string for the document
    pub fn topic(&self, subtopic: &str) -> String {
        ["documents:", &self.id, ":", subtopic].concat()
    }

    /// Subscribe a client to one of the document's topics
    pub fn subscribe(&mut self, topic: &str, client: &str) -> String {
        match self.subscriptions.entry(topic.into()) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().insert(client.into());
            }
            Entry::Vacant(vacant) => {
                vacant.insert(hashset! {client.into()});
            }
        }
        self.topic(topic)
    }

    /// Unsubscribe a client from one of the document's topics
    pub fn unsubscribe(&mut self, topic: &str, client: &str) -> String {
        if let Entry::Occupied(mut occupied) = self.subscriptions.entry(topic.to_string()) {
            let subscribers = occupied.get_mut();
            subscribers.remove(client);
            if subscribers.is_empty() {
                occupied.remove();
            }
        }
        self.topic(topic)
    }

    /// Get the number of subscribers to one of the document's topics
    fn subscribers(&self, topic: &str) -> usize {
        if let Some(subscriptions) = self.subscriptions.get(topic) {
            subscriptions.len()
        } else {
            0
        }
    }

    /// Publish an event for this document
    fn publish(&self, type_: DocumentEventType, content: Option<String>, format: Option<String>) {
        let format = format.map(|name| formats::match_name(&name).spec());

        let subtopic = match type_ {
            DocumentEventType::Encoded => format!(
                "encoded:{}",
                format
                    .clone()
                    .map_or_else(|| "undef".to_string(), |format| format.extension)
            ),
            _ => type_.to_string(),
        };

        publish(
            &self.topic(&subtopic),
            &DocumentEvent {
                type_,
                document: self.repr(),
                content,
                format,
                patch: None,
            },
        )
    }

    /// Called when the file is removed from the file system
    ///
    /// Sets `status` to `Deleted` and publishes a `Deleted` event so that,
    /// for example, a document's tab can be updated to indicate it is deleted.
    fn deleted(&mut self, path: PathBuf) {
        tracing::debug!(
            "Deleted event for document `{}` at `{}`",
            self.id,
            path.display()
        );

        self.status = DocumentStatus::Deleted;

        self.publish(DocumentEventType::Deleted, None, None)
    }

    /// Called when the file is renamed
    ///
    /// Changes the `path` and publishes a `Renamed` event so that, for example,
    /// a document's tab can be updated with the new file name.
    #[allow(dead_code)]
    fn renamed(&mut self, from: PathBuf, to: PathBuf) {
        tracing::debug!(
            "Renamed event for document `{}`: `{}` to `{}`",
            self.id,
            from.display(),
            to.display()
        );

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
    /// Will ignore any events within a small duration of `write()` being called to avoid
    /// reacting to file modifications initiated by this process
    async fn modified(&mut self, path: PathBuf) {
        if let Some(last_write) = self.last_write {
            if last_write.elapsed() < Duration::from_millis(Document::LAST_WRITE_MUTE_MILLIS) {
                return;
            }
        }

        tracing::debug!(
            "Modified event for document `{}` at `{}`",
            self.id,
            path.display()
        );

        self.status = DocumentStatus::Unread;

        match self.read(false).await {
            Ok(content) => self.publish(
                DocumentEventType::Modified,
                Some(content),
                Some(self.format.extension.clone()),
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
#[derive(Debug, Default)]
pub struct Documents {
    /// A mapping of file paths to open documents
    registry: Mutex<HashMap<String, DocumentHandler>>,
}

impl Documents {
    /// Create a new documents store
    pub fn new() -> Self {
        Self::default()
    }

    /// List documents that are currently open
    ///
    /// Returns a vector of document paths (relative to the current working directory)
    pub async fn list(&self) -> Result<Vec<String>> {
        let cwd = std::env::current_dir()?;
        let mut paths = Vec::new();
        for document in self.registry.lock().await.values() {
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
    pub async fn create(&self, path: Option<String>, format: Option<String>) -> Result<Document> {
        let path = path.map(PathBuf::from);

        let document = Document::new(path, format);
        let document_id = document.id.clone();
        let document_repr = document.repr();
        let handler = DocumentHandler::new(document, false);
        self.registry.lock().await.insert(document_id, handler);

        Ok(document_repr)
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
    pub async fn open<P: AsRef<Path>>(&self, path: P, format: Option<String>) -> Result<Document> {
        let path = Path::new(path.as_ref()).canonicalize()?;

        for handler in self.registry.lock().await.values() {
            let document = handler.document.lock().await;
            if document.path == path {
                return Ok(document.repr());
            }
        }

        let document = Document::open(path, format).await?;
        let document_id = document.id.clone();
        let document_repr = document.repr();
        let handler = DocumentHandler::new(document, true);
        self.registry.lock().await.insert(document_id, handler);

        Ok(document_repr)
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
    pub async fn close<P: AsRef<Path>>(&self, id_or_path: P) -> Result<()> {
        let id_or_path_path = id_or_path.as_ref();
        let id_or_path_string = id_or_path_path.to_string_lossy().to_string();
        let mut id_to_remove = String::new();

        if self.registry.lock().await.contains_key(&id_or_path_string) {
            id_to_remove = id_or_path_string
        } else {
            let path = id_or_path_path.canonicalize()?;
            for handler in self.registry.lock().await.values() {
                let document = handler.document.lock().await;
                if document.path == path {
                    id_to_remove = document.id.clone();
                    break;
                }
            }
        };
        self.registry.lock().await.remove(&id_to_remove);

        Ok(())
    }

    /// Subscribe a client to a topic for a document
    pub async fn subscribe(
        &self,
        id: &str,
        topic: &str,
        client: &str,
    ) -> Result<(Document, String)> {
        let document_lock = self.get(id).await?;
        let mut document_guard = document_lock.lock().await;
        let topic = document_guard.subscribe(topic, client);
        Ok((document_guard.repr(), topic))
    }

    /// Unsubscribe a client from a topic for a document
    pub async fn unsubscribe(
        &self,
        id: &str,
        topic: &str,
        client: &str,
    ) -> Result<(Document, String)> {
        let document_lock = self.get(id).await?;
        let mut document_guard = document_lock.lock().await;
        let topic = document_guard.unsubscribe(topic, client);
        Ok((document_guard.repr(), topic))
    }

    /// Patch a document
    ///
    /// Given that this function is likely to be called often, to avoid a `clone()` and
    /// to reduce WebSocket message sizes, unlike other functions it does not return the object.
    #[tracing::instrument(skip(self))]
    pub async fn patch(&self, id: &str, patch: Patch) -> Result<()> {
        let document_lock = self.get(id).await?;
        let mut document_guard = document_lock.lock().await;
        document_guard.patch(patch).await
    }

    /// Execute a node within a document
    ///
    /// Like `patch()`, given this function is likely to be called often, do not return
    /// the document.
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, id: &str, start: Option<String>) -> Result<()> {
        let document_lock = self.get(id).await?;
        let mut document_guard = document_lock.lock().await;
        document_guard.execute(start).await
    }

    /// Get a document that has previously been opened
    pub async fn get(&self, id: &str) -> Result<Arc<Mutex<Document>>> {
        if let Some(handler) = self.registry.lock().await.get(id) {
            Ok(handler.document.clone())
        } else {
            bail!("No document with id {}", id)
        }
    }
}

/// The global documents store
pub static DOCUMENTS: Lazy<Documents> = Lazy::new(Documents::new);

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![
        schemas::generate::<Document>()?,
        schemas::generate::<DocumentEvent>()?,
    ]);
    Ok(schemas)
}

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use crate::utils::json;
    use async_trait::async_trait;
    use cli_utils::{result, Result, Run};
    use node_patch::diff_display;
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

        Execute(Execute),
        Kernels(Kernels),
        Tasks(Tasks),
        Queues(Queues),
        Cancel(Cancel),
        Symbols(Symbols),
        Graph(Graph),

        Query(Query),
        Convert(Convert),
        Diff(Diff),
        Merge(Merge),
        Schemas(Schemas),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::List(action) => action.run().await,
                Action::Open(action) => action.run().await,
                Action::Close(action) => action.run().await,
                Action::Show(action) => action.run().await,

                Action::Execute(action) => action.run().await,
                Action::Kernels(action) => action.run().await,
                Action::Tasks(action) => action.run().await,
                Action::Queues(action) => action.run().await,
                Action::Cancel(action) => action.run().await,
                Action::Symbols(action) => action.run().await,
                Action::Graph(action) => action.run().await,

                Action::Query(action) => action.run().await,
                Action::Convert(action) => action.run().await,
                Action::Diff(action) => action.run().await,
                Action::Merge(action) => action.run().await,
                Action::Schemas(action) => action.run(),
            }
        }
    }

    // The arguments used to specify the document file path and format
    // Reused (with flatten) below
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    struct File {
        /// The path of the document file
        path: String,

        /// The format of the document file
        #[structopt(short, long)]
        format: Option<String>,
    }
    impl File {
        async fn open(&self) -> eyre::Result<Document> {
            DOCUMENTS.open(&self.path, self.format.clone()).await
        }

        async fn get(&self) -> eyre::Result<Arc<Mutex<Document>>> {
            let document = self.open().await?;
            DOCUMENTS.get(&document.id).await
        }
    }

    /// List open documents
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}
    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let list = DOCUMENTS.list().await?;
            result::value(list)
        }
    }

    /// Open a document
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Open {
        #[structopt(flatten)]
        file: File,
    }
    #[async_trait]
    impl Run for Open {
        async fn run(&self) -> Result {
            self.file.open().await?;
            result::nothing()
        }
    }

    /// Close a document
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Close {
        /// The path of the document file
        pub path: String,
    }
    #[async_trait]
    impl Run for Close {
        async fn run(&self) -> Result {
            DOCUMENTS.close(&self.path).await?;
            result::nothing()
        }
    }

    /// Show a document
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        #[structopt(flatten)]
        file: File,

        /// A pointer to the part of the document to show e.g. `variables`, `format.name`
        ///
        /// Some, usually large, document properties are only shown when specified with a
        /// pointer (e.g. `content` and `root`).
        pub pointer: Option<String>,
    }
    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let document = self.file.open().await?;
            if let Some(pointer) = &self.pointer {
                if pointer == "content" {
                    result::content(&document.format.extension, &document.content)
                } else if pointer == "root" {
                    let root = &*document.root.read().await;
                    result::value(root)
                } else {
                    let data = serde_json::to_value(document)?;
                    if let Some(part) = data.pointer(&json::pointer(pointer)) {
                        Ok(result::value(part)?)
                    } else {
                        bail!("Invalid pointer for document: {}", pointer)
                    }
                }
            } else {
                result::value(document)
            }
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        alias = "exec",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Execute {
        #[structopt(flatten)]
        file: File,

        #[structopt(flatten)]
        execute: kernels::commands::Execute,
    }
    #[async_trait]
    impl Run for Execute {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let _kernels = document.kernels.clone();
            //self.execute.run(&mut kernels).await
            result::nothing()
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Kernels {
        #[structopt(flatten)]
        file: File,

        #[structopt(flatten)]
        kernels: kernels::commands::Running,
    }
    #[async_trait]
    impl Run for Kernels {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let kernels = document.kernels.clone();
            self.kernels.run(&*kernels).await
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Tasks {
        #[structopt(flatten)]
        file: File,

        #[structopt(flatten)]
        tasks: kernels::commands::Tasks,
    }
    #[async_trait]
    impl Run for Tasks {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let kernels = document.kernels.clone();
            self.tasks.run(&*kernels).await
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Queues {
        #[structopt(flatten)]
        file: File,

        #[structopt(flatten)]
        queues: kernels::commands::Queues,
    }
    #[async_trait]
    impl Run for Queues {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let kernels = document.kernels.clone();
            self.queues.run(&kernels).await
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Cancel {
        #[structopt(flatten)]
        file: File,

        #[structopt(flatten)]
        cancel: kernels::commands::Cancel,
    }
    #[async_trait]
    impl Run for Cancel {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let _kernels = document.kernels.clone();
            //self.cancel.run(&mut *kernels).await
            result::nothing()
        }
    }
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Symbols {
        #[structopt(flatten)]
        file: File,

        #[structopt(flatten)]
        symbols: kernels::commands::Symbols,
    }
    #[async_trait]
    impl Run for Symbols {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let kernels = document.kernels.clone();
            self.symbols.run(&kernels).await
        }
    }

    /// Output the dependency graph for a document
    ///
    /// Tip: When using the DOT format (the default), if you have GraphViz and ImageMagick
    /// installed you can view the graph by piping the output to them. For example, to
    /// view a graph of the current project:
    ///
    /// ```sh
    /// stencila documents graph | dot -Tpng | display
    /// ```
    ///
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Graph {
        #[structopt(flatten)]
        file: File,

        /// The format to output the graph as
        #[structopt(long, short, default_value = "dot", possible_values = &graph::FORMATS)]
        r#as: String,
    }

    #[async_trait]
    impl Run for Graph {
        async fn run(&self) -> Result {
            let document = self.file.get().await?;
            let document = document.lock().await;
            let content = document.graph.to_format(&self.r#as)?;
            result::content(&self.r#as, &content)
        }
    }

    /// Query a document
    #[derive(Debug, StructOpt)]
    #[structopt(
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
    #[async_trait]
    impl Run for Query {
        async fn run(&self) -> Result {
            let Self {
                file,
                format,
                query,
                lang,
            } = self;
            let document = DOCUMENTS.open(file, format.clone()).await?;
            let result = document.query(query, lang).await?;
            result::value(result)
        }
    }

    /// Convert a document to another format
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Convert {
        /// The path of the input document
        pub input: PathBuf,

        /// The path of the output document
        ///
        /// Use `-` to print output to the console's standard output.
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
    #[async_trait]
    impl Run for Convert {
        async fn run(&self) -> Result {
            let document = Document::open(&self.input, self.from.clone()).await?;

            let out = self.output.display().to_string();
            if out == "-" {
                let format = match &self.to {
                    None => "json".to_string(),
                    Some(format) => format.clone(),
                };
                let content = document.dump(Some(format.clone())).await?;
                result::content(&format, &content)
            } else {
                document
                    .write_as(&self.output, self.to.clone(), self.theme.clone())
                    .await?;
                result::nothing()
            }
        }
    }

    /// Display the structural differences between two documents
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Diff {
        /// The path of the first document
        first: PathBuf,

        /// The path of the second document
        second: PathBuf,

        /// The format to display the difference in
        ///
        /// Defaults to a "unified diff" of the JSON representation
        /// of the documents. Unified diffs of other formats are available
        /// e.g. "md", "yaml". Use "raw" for the raw patch as a list of
        /// operations.
        #[structopt(short, long, default_value = "json")]
        format: String,
    }
    #[async_trait]
    impl Run for Diff {
        async fn run(&self) -> Result {
            let Self {
                first,
                second,
                format,
            } = self;
            let first = Document::open(first, None).await?;
            let second = Document::open(second, None).await?;

            let first = &*first.root.read().await;
            let second = &*second.root.read().await;

            if format == "raw" {
                let patch = diff(first, second);
                result::value(patch)
            } else {
                let diff = diff_display(first, second, format).await?;
                result::content("patch", &diff)
            }
        }
    }

    /// Merge changes from two or more derived versions of a document
    ///
    /// This command can be used as a Git custom "merge driver".
    /// First, register Stencila as a merge driver,
    ///
    /// git config merge.stencila.driver "stencila merge --git %O %A %B"
    ///
    /// (The placeholders `%A` etc are used by `git` to pass arguments such
    /// as file paths and options to `stencila`.)
    ///
    /// Then, in your `.gitattributes` file assign the driver to specific
    /// types of files e.g.,
    ///
    /// *.{md|docx} merge=stencila
    ///
    /// This can be done per project, or globally.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    // See https://git-scm.com/docs/gitattributes#_defining_a_custom_merge_driver and
    // https://www.julianburr.de/til/custom-git-merge-drivers/ for more examples of defining a
    // custom driver. In particular the meaning of the placeholders %O, %A etc
    pub struct Merge {
        /// The path of the original version
        original: PathBuf,

        /// The paths of the derived versions
        #[structopt(required = true, multiple = true)]
        derived: Vec<PathBuf>,

        /// A flag to indicate that the command is being used as a Git merge driver
        ///
        /// When the `merge` command is used as a Git merge driver the second path
        /// supplied is the file that is written to.
        #[structopt(short, long)]
        git: bool,
    }
    #[async_trait]
    impl Run for Merge {
        async fn run(&self) -> Result {
            let mut original = Document::open(&self.original, None).await?;

            let mut docs: Vec<Document> = Vec::new();
            for path in &self.derived {
                docs.push(Document::open(path, None).await?)
            }

            original.merge(&docs).await?;

            if self.git {
                original.write_as(&self.derived[0], None, None).await?;
            } else {
                original.write(None, None).await?;
            }

            result::nothing()
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Get JSON Schemas for documents and associated types",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Schemas {}

    impl Schemas {
        pub fn run(&self) -> Result {
            let schema = schemas()?;
            result::value(schema)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use test_utils::fixtures;

    #[test]
    fn new() {
        let doc = Document::new(None, None);
        assert!(doc.path.starts_with(env::temp_dir()));
        assert!(doc.temporary);
        assert!(matches!(doc.status, DocumentStatus::Synced));
        assert_eq!(doc.format.extension, "txt");
        assert_eq!(doc.content, "");
        assert_eq!(doc.subscriptions, hashmap! {});

        let doc = Document::new(None, Some("md".to_string()));
        assert!(doc.path.starts_with(env::temp_dir()));
        assert!(doc.temporary);
        assert!(matches!(doc.status, DocumentStatus::Synced));
        assert_eq!(doc.format.extension, "md");
        assert_eq!(doc.content, "");
        assert_eq!(doc.subscriptions, hashmap! {});
    }

    #[tokio::test]
    async fn open() -> Result<()> {
        for file in &["elife-small.json", "era-plotly.json"] {
            let doc = Document::open(fixtures().join("articles").join(file), None).await?;
            assert!(!doc.temporary);
            assert!(matches!(doc.status, DocumentStatus::Synced));
            assert_eq!(doc.format.extension, "json");
            assert!(!doc.content.is_empty());
            assert_eq!(doc.subscriptions, hashmap! {});
        }

        Ok(())
    }
}
