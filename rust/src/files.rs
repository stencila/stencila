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

use crate::pubsub::{self, ProjectFileEvent};

/// # A file or directory within a `Project`
#[skip_serializing_none]
#[derive(Debug, Default, Clone, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
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

/// A registry of `File`s within a `Project` including ignore files
#[derive(Debug, Default, JsonSchema, Serialize)]
pub struct FileRegistry {
    #[serde(skip)]
    path: PathBuf,

    #[serde(flatten)]
    pub files: BTreeMap<PathBuf, File>,

    #[serde(skip)]
    ignores: BTreeSet<PathBuf>,
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
            .build();

        // Collect files
        let mut files = walker
            .into_iter()
            .filter_map(|entry| {
                let entry = match entry.ok() {
                    Some(entry) => entry,
                    None => return None,
                };
                File::load(entry.path()).ok()
            })
            .into_iter()
            .collect::<BTreeMap<PathBuf, File>>();

        // Resolve `children` properties and read any `ignore` files
        let mut ignores = BTreeSet::new();
        for path in files.keys().cloned().collect::<Vec<PathBuf>>() {
            if FileRegistry::is_gitignore(&path) {
                ignores.insert(path.clone());
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
            ignores,
        }
    }

    /// Refresh the file registry
    fn refresh(&mut self) {
        *self = FileRegistry::new(self.path.as_path());
    }

    /// Should the registry be refreshed in response to a change in a file
    ///
    /// For example if a `.gitignore` file is added, removed, moved or modified.
    fn should_refresh(&mut self, path: &Path) -> bool {
        FileRegistry::is_gitignore(&path)
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
    fn is_gitignore(path: &Path) -> bool {
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
    /// Checks against any of the `ignores` files that are "above" the file.
    fn should_ignore(&self, path: &Path) -> bool {
        for ignore_file_path in &self.ignores {
            if let Some(ignore_file_dir) = ignore_file_path.parent() {
                if path.starts_with(ignore_file_dir) {
                    if let Ok(ignore_file) = gitignore::File::new(&ignore_file_path) {
                        return ignore_file.is_excluded(path).unwrap_or(false);
                    }
                }
            }
        }
        false
    }

    // Update a project file registry when a file is created
    pub fn created(&mut self, project: &Path, path: &Path) {
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

        pubsub::publish_project_file(ProjectFileEvent {
            project: project.into(),
            path: path.into(),
            kind: "created".into(),
            file,
            files: Some(self.files.clone()),
        })
    }

    // Update a project file registry when a file is removed
    pub fn removed(&mut self, project: &Path, path: &Path) {
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

        pubsub::publish_project_file(ProjectFileEvent {
            project: project.into(),
            path: path.into(),
            kind: "removed".into(),
            file: None,
            files: Some(self.files.clone()),
        })
    }

    // Update a project file registry when a file is renamed
    pub fn renamed(&mut self, project: &Path, old_path: &Path, new_path: &Path) {
        if self.should_refresh(old_path) || self.should_refresh(new_path) {
            return self.refresh();
        }

        let ignore_old = self.should_ignore(old_path);
        let ignore_new = self.should_ignore(new_path);
        if ignore_old && ignore_new {
            return;
        } else if ignore_new {
            return self.removed(project, old_path);
        } else if ignore_old {
            return self.created(project, new_path);
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

        pubsub::publish_project_file(ProjectFileEvent {
            project: project.into(),
            path: old_path.into(),
            kind: "renamed".into(),
            file,
            files: Some(self.files.clone()),
        })
    }

    // Update a project file registry when a file is modified
    pub fn modified(&mut self, project: &Path, path: &Path) {
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

        pubsub::publish_project_file(ProjectFileEvent {
            project: project.into(),
            path: path.into(),
            kind: "modified".into(),
            file,
            files: Some(self.files.clone()),
        })
    }
}

/// # The set of `File`s within a `Project`
#[derive(Debug, Default, Clone, JsonSchema, Serialize)]
pub struct Files {
    #[serde(flatten)]
    pub registry: Arc<Mutex<FileRegistry>>,

    #[serde(skip)]
    pub watcher: Option<std::sync::mpsc::Sender<()>>,
}

impl Files {
    /// Load files from a folder
    pub fn load(folder: &str, watch: bool) -> Result<Files> {
        let path = Path::new(folder).canonicalize()?;

        // Create a registry of the files
        let registry = Arc::new(Mutex::new(FileRegistry::new(&path)));

        // Watch files and make updates as needed
        let watcher = if watch {
            let project = path.clone();
            let registry = Arc::clone(&registry);
            let (thread_sender, thread_receiver) = channel();
            std::thread::spawn(move || -> Result<()> {
                use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
                use std::time::Duration;

                let (watcher_sender, watcher_receiver) = channel();
                let mut watcher = watcher(watcher_sender, Duration::from_secs(1))?;
                watcher.watch(&path, RecursiveMode::Recursive).unwrap();

                let handle_event = |event| {
                    let registry = &mut *registry.lock().unwrap();
                    match event {
                        DebouncedEvent::Create(path) => registry.created(&project, &path),
                        DebouncedEvent::Remove(path) => registry.removed(&project, &path),
                        DebouncedEvent::Rename(from, to) => registry.renamed(&project, &from, &to),
                        DebouncedEvent::Write(path) => registry.modified(&project, &path),
                        _ => {}
                    }
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
