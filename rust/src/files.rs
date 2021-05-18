use eyre::{bail, Result};
use schemars::JsonSchema;
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, TryRecvError},
        Arc, Mutex, MutexGuard,
    },
    time::UNIX_EPOCH,
};

use crate::pubsub::publish;

/// A file or directory within a `Project`
#[skip_serializing_none]
#[derive(Debug, Default, Clone, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[schemars(title = "File", deny_unknown_fields)]
pub struct File {
    /// The absolute path of the file or directory
    pub path: PathBuf,

    /// The name of the file or directory
    pub name: String,

    /// Time that the file was last modified (Unix Epoch timestamp)
    pub modified: Option<u64>,

    /// Size of the file in bytes
    pub size: Option<u64>,

    /// Format of the file
    ///
    /// Usually this is the lower cased filename extension (if any)
    /// but may also be normalized. May be more convenient,
    /// and usually more available, than the `media_type` property.
    pub format: Option<String>,

    /// The media type (aka MIME type) of the file
    pub media_type: Option<String>,

    /// The SHA1 hash of the contents of the file
    pub sha1: Option<String>,

    /// The parent `File`, if any
    pub parent: Option<PathBuf>,

    /// If a directory, a list of the canonical paths of the files within it.
    /// Otherwise, `None`.
    ///
    /// A `BTreeSet` rather than a `Vec` so that paths are ordered without
    /// having to be resorted after insertions. Another option is `BinaryHeap`
    /// but `BinaryHeap::retain` is  only on nightly and so is awkward to use.
    pub children: Option<BTreeSet<PathBuf>>,
}

impl File {
    // Load a file from a path
    pub fn load(path: &Path) -> Result<(PathBuf, File)> {
        let path = path.canonicalize()?;

        let name = path
            .file_name()
            .map(|os_str| os_str.to_string_lossy())
            .unwrap_or_default()
            .into();

        let (modified, size) = match path.metadata() {
            Ok(metadata) => {
                #[allow(clippy::bind_instead_of_map)]
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .and_then(|duration| Some(duration.as_secs()));
                let size = Some(metadata.len());
                (modified, size)
            }
            Err(_) => (None, None),
        };

        let format = path
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase());

        let (media_type, children) = if path.is_file() {
            let media_type = if let Some(ext) = &format {
                mime_guess::from_ext(&ext)
                    .first()
                    .map(|mime| mime.essence_str().to_string())
                    .or_else(|| match ext.as_str() {
                        // Add MIME types that are not registered
                        // See https://github.com/jupyter/jupyter/issues/68
                        "ipynb" => Some("application/ipynb+json".to_string()),
                        _ => None,
                    })
            } else {
                None
            };

            (media_type, None)
        } else {
            (None, Some(BTreeSet::new()))
        };

        let file = File {
            path,
            name,
            modified,
            size,
            format,
            media_type,
            children,
            ..Default::default()
        };

        Ok((file.path.clone(), file))
    }
}

/// An event associated with a `File` or a set of `File`s
///
/// These events published under the `project:<project-path>:file` topic.
/// Specific child topics include:
///
/// - `project:<project-path>:file:*:refreshed`
/// - `project:<project-path>:file:<file-path>:created`
/// - `project:<project-path>:file:<file-path>:removed`
/// - `project:<project-path>:file:<file-path>:renamed`
/// - `project:<project-path>:file:<file-path>:modified`
///
#[derive(Debug, Serialize)]
pub struct FileEvent {
    /// The path of the project (absolute)
    pub project: PathBuf,

    /// The path of the file (absolute)
    ///
    /// For `renamed` events this is the _old_ path.
    pub path: PathBuf,

    /// The kind of event e.g. `refreshed`, `modified`, `created`
    ///
    /// A `refreshed` event is emitted when the entire set of
    /// files is updated.
    pub kind: String,

    /// The updated file
    ///
    /// Will be `None` for for `refreshed` and `removed` events,
    /// or if for some reason it was not possible to fetch metadata
    /// about the file.
    pub file: Option<File>,

    /// The updated set of files in the project
    ///
    /// Represents the new state of the file tree after the
    /// event including updated `parent` and `children`
    /// properties of files affects by the event.
    pub files: BTreeMap<PathBuf, File>,
}

/// A registry of `File`s within a `Project`
#[derive(Debug, Default, JsonSchema, Serialize)]
pub struct FileRegistry {
    /// The root path of the project
    #[serde(skip)]
    path: PathBuf,

    /// The map of files in the project
    #[serde(flatten)]
    pub files: BTreeMap<PathBuf, File>,

    /// The set of Git ignore style files in the project
    ///
    /// Used to avoid adding ignored file when notified
    /// of changes by the watcher thread.
    #[serde(skip)]
    ignore_files: BTreeSet<PathBuf>,

    /// The set of files that, according to `ignore_files`
    /// should be ignored.
    ///
    /// Used as a cache to avoid reading and processing
    /// ignore files when notified of changes by the
    /// watcher thread.
    #[serde(skip)]
    files_ignored: BTreeSet<PathBuf>,
}

impl FileRegistry {
    const GITIGNORE_NAMES: [&'static str; 2] = [".ignore", ".gitignore"];

    pub fn new(path: &Path) -> FileRegistry {
        // Build walker
        let walker = ignore::WalkBuilder::new(&path)
            // Consider .ignore files
            .ignore(true)
            // Consider .gitignore files
            .git_ignore(true)
            .build_parallel();

        // Collect files in parallel using a collector thread and several walker thread
        // (number of which is chosen by the `ignore` walker)
        let (sender, receiver) = channel();
        let join_handle =
            std::thread::spawn(move || -> BTreeMap<PathBuf, File> { receiver.iter().collect() });
        walker.run(|| {
            let sender = sender.clone();
            Box::new(move |result| {
                use ignore::WalkState::*;

                if let Ok(entry) = result {
                    if let Ok(file) = File::load(entry.path()) {
                        sender.send(file).expect("Unable to send to collector");
                    }
                }

                Continue
            })
        });
        drop(sender);
        let mut files = join_handle.join().expect("Unable to join collector thread");

        // Resolve `children` properties and `ignore_files` files
        let mut ignore_files = BTreeSet::new();
        for path in files.keys().cloned().collect::<Vec<PathBuf>>() {
            if FileRegistry::is_ignore_file(&path) {
                ignore_files.insert(path.clone());
            }

            if let Some(parent) = path.parent() {
                if let Entry::Occupied(mut parent) = files.entry(parent.into()) {
                    let parent = parent.get_mut();
                    if let Some(children) = &mut parent.children {
                        children.insert(path);
                    }
                }
            }
        }

        let path = path.to_path_buf();
        FileRegistry {
            path,
            files,
            ignore_files,
            ..Default::default()
        }
    }

    /// Publish a `FileEvent` under the project's topic
    pub fn publish_file_event(&self, path: &Path, kind: &str, file: Option<File>) {
        let topic = &format!(
            "project:{}:file:{}:{}",
            self.path.display(),
            path.display(),
            kind
        );
        let event = FileEvent {
            project: self.path.clone(),
            path: path.into(),
            kind: "created".into(),
            file,
            files: self.files.clone(),
        };
        publish(topic, &event)
    }

    /// Should the registry be refreshed in response to a change in a file
    ///
    /// For example if a `.gitignore` file is added, removed, moved or modified.
    fn should_refresh(&mut self, path: &Path) -> bool {
        FileRegistry::is_ignore_file(&path)
    }

    /// Refresh the registry if it should be
    fn did_refresh(&mut self, path: &Path) -> bool {
        if self.should_refresh(&path) {
            self.refresh();
            true
        } else {
            false
        }
    }

    /// Is the file a Git ignore file?
    fn is_ignore_file(path: &Path) -> bool {
        if let Some(name) = path
            .file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
        {
            if FileRegistry::GITIGNORE_NAMES.contains(&name.as_str()) {
                return true;
            }
        }
        false
    }

    /// Should a path be ignored?
    ///
    /// Used by the following functions to decide whether to update a file
    /// in the registry. Tries to be consistent with the `ignore` crate (which
    /// is used to initially load all the files).
    ///
    /// Checks against any of the `ignore_files` that are "above" the file in
    /// the file tree. Caches result to minimize re-reading the ignore file.
    fn should_ignore(&mut self, path: &Path) -> bool {
        if self.files_ignored.contains(path) {
            return true;
        }

        for ignore_file_path in &self.ignore_files {
            if let Some(ignore_file_dir) = ignore_file_path.parent() {
                if path.starts_with(ignore_file_dir) {
                    if let Ok(ignore_file) = gitignore::File::new(&ignore_file_path) {
                        if ignore_file.is_excluded(path).unwrap_or(false) {
                            self.files_ignored.insert(path.into());
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Refresh the file registry
    fn refresh(&mut self) {
        *self = FileRegistry::new(self.path.as_path());

        self.publish_file_event(Path::new("*"), "refresh", None)
    }

    // Update a project file registry when a file is created
    pub fn created(&mut self, path: &Path) {
        if self.should_ignore(path) || self.did_refresh(path) {
            return;
        }

        // Load the file, insert it and add it to it's parent's children
        let file = if let Ok((path, file)) = File::load(path) {
            self.files.insert(path.clone(), file.clone());
            if let Some(parent) = path.parent().and_then(|parent| self.files.get_mut(parent)) {
                if let Some(children) = &mut parent.children {
                    children.insert(path);
                }
            }

            Some(file)
        } else {
            None
        };

        self.publish_file_event(path, "created", file)
    }

    // Update a project file registry when a file is removed
    pub fn removed(&mut self, path: &Path) {
        if self.should_ignore(path) || self.did_refresh(path) {
            return;
        }

        // Remove the file and remove it from its parent's children
        self.files.remove(path);
        if let Some(parent) = path.parent().and_then(|parent| self.files.get_mut(parent)) {
            if let Some(children) = &mut parent.children {
                children.remove(path);
            }
        }

        self.publish_file_event(path, "removed", None)
    }

    // Update a project file registry when a file is renamed
    pub fn renamed(&mut self, old_path: &Path, new_path: &Path) {
        if self.should_refresh(old_path) || self.should_refresh(new_path) {
            return self.refresh();
        }

        let ignore_old = self.should_ignore(old_path);
        let ignore_new = self.should_ignore(new_path);
        if ignore_old && ignore_new {
            return;
        } else if ignore_new {
            return self.removed(old_path);
        } else if ignore_old {
            return self.created(new_path);
        }

        // Move the file
        let file = match self.files.entry(old_path.into()) {
            Entry::Occupied(entry) => {
                // Update it's path and parent properties, and the paths of
                // all it's descendants. Move the file from old to new path.
                let mut file = entry.remove();
                file.path = new_path.into();
                file.parent = new_path.parent().map(|parent| parent.into());
                rename_children(&mut self.files, &mut file, old_path, new_path);
                self.files.insert(new_path.into(), file.clone());
                Some(file)
            }
            Entry::Vacant(_) => {
                // The entry should not be empty, but in case it is, load the file from,
                // and insert it at, the new path
                if let Ok((_path, file)) = File::load(new_path) {
                    self.files.insert(new_path.into(), file.clone());
                    Some(file)
                } else {
                    None
                }
            }
        };

        // Recursively rename children of a file
        fn rename_children(
            registry: &mut BTreeMap<PathBuf, File>,
            file: &mut File,
            old_path: &Path,
            new_path: &Path,
        ) {
            if let Some(children) = &mut file.children {
                let mut new_children = BTreeSet::new();
                for child_old_path in children.iter() {
                    let child_new_path = new_path.join(
                        child_old_path
                            .strip_prefix(old_path)
                            .expect("Unable to strip old path"),
                    );

                    if let Entry::Occupied(entry) = registry.entry(child_old_path.into()) {
                        let mut file = entry.remove();
                        file.path = child_new_path.clone();
                        file.parent = child_new_path.parent().map(|parent| parent.into());
                        rename_children(registry, &mut file, child_old_path, &child_new_path);
                        registry.insert(child_new_path.clone(), file);
                    }

                    new_children.insert(child_new_path);
                }
                file.children = Some(new_children);
            }
        }

        // Remove the old path from the old parent's children
        if let Some(parent) = old_path
            .parent()
            .and_then(|parent| self.files.get_mut(parent))
        {
            if let Some(children) = &mut parent.children {
                children.remove(old_path);
            }
        }

        // Insert the new path to the new parent's children
        if let Some(parent) = new_path
            .parent()
            .and_then(|parent| self.files.get_mut(parent))
        {
            if let Some(children) = &mut parent.children {
                children.insert(new_path.into());
            }
        }

        self.publish_file_event(old_path, "renamed", file)
    }

    // Update a project file registry when a file is modified
    pub fn modified(&mut self, path: &Path) {
        if self.should_ignore(path) || self.did_refresh(path) {
            return;
        }

        // Insert the file
        let file = if let Ok((path, file)) = File::load(path) {
            self.files.insert(path, file.clone());
            Some(file)
        } else {
            None
        };

        self.publish_file_event(path, "modified", file)
    }
}

/// The set of `File`s within a `Project`
#[derive(Debug, Default, Clone, JsonSchema, Serialize)]
pub struct Files {
    #[serde(flatten)]
    pub registry: Arc<Mutex<FileRegistry>>,

    #[serde(skip)]
    pub watcher: Option<std::sync::mpsc::Sender<()>>,
}

impl Files {
    /// Load files from a folder
    pub fn load(folder: &str, watch: bool, watch_exclude_patterns: Vec<String>) -> Result<Files> {
        let path = Path::new(folder).canonicalize()?;

        // Create a registry of the files
        let registry = Arc::new(Mutex::new(FileRegistry::new(&path)));

        // Watch files and make updates as needed
        let watcher = if watch {
            let registry = Arc::clone(&registry);
            let (thread_sender, thread_receiver) = channel();
            std::thread::spawn(move || -> Result<()> {
                use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
                use std::time::Duration;

                let (watcher_sender, watcher_receiver) = channel();
                let mut watcher = watcher(watcher_sender, Duration::from_secs(1))?;
                watcher.watch(&path, RecursiveMode::Recursive).unwrap();

                let exclude_globs: Vec<glob::Pattern> = watch_exclude_patterns
                    .iter()
                    .filter_map(|pattern| match glob::Pattern::new(pattern) {
                        Ok(glob) => Some(glob),
                        Err(error) => {
                            tracing::warn!(
                                "Invalid watch exclude glob pattern; will ignore: {} : {}",
                                pattern,
                                error
                            );
                            None
                        }
                    })
                    .collect();

                let should_include = |event_path: &PathBuf| {
                    if let Ok(event_path) = event_path.strip_prefix(&path) {
                        for glob in &exclude_globs {
                            if glob.matches(&event_path.display().to_string()) {
                                return false;
                            }
                        }
                    }
                    true
                };

                let handle_event = |event| match event {
                    DebouncedEvent::Create(path) => {
                        if should_include(&path) {
                            let registry = &mut *registry.lock().unwrap();
                            registry.created(&path)
                        }
                    }
                    DebouncedEvent::Remove(path) => {
                        if should_include(&path) {
                            let registry = &mut *registry.lock().unwrap();
                            registry.modified(&path)
                        }
                    }
                    DebouncedEvent::Rename(from, to) => {
                        if should_include(&from) || should_include(&to) {
                            let registry = &mut *registry.lock().unwrap();
                            registry.renamed(&from, &to);
                        }
                    }
                    DebouncedEvent::Write(path) => {
                        if should_include(&path) {
                            let registry = &mut *registry.lock().unwrap();
                            registry.modified(&path)
                        }
                    }
                    _ => {}
                };

                let project = path.display().to_string();
                let span = tracing::info_span!("file_watch", project = project.as_str());
                let _enter = span.enter();
                tracing::debug!("Starting project file watch: {}", project);

                loop {
                    if let Err(TryRecvError::Disconnected) = thread_receiver.try_recv() {
                        tracing::debug!("Ending project file watch: {}", project);
                        break;
                    }
                    match watcher_receiver.recv() {
                        Ok(event) => handle_event(event),
                        Err(error) => tracing::error!("Watch error: {:?}", error),
                    }
                }

                Ok(())
            });

            Some(thread_sender)
        } else {
            None
        };

        let files = Files { registry, watcher };
        Ok(files)
    }

    // Obtain the file registry
    pub fn registry(&self) -> Result<MutexGuard<FileRegistry>> {
        match self.registry.try_lock() {
            Ok(registry) => Ok(registry),
            Err(error) => bail!("Unable to get file registry: {}", error),
        }
    }
}
