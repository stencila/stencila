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

/// # The set of `File`s within a `Project`
#[derive(Debug, Default, Clone, JsonSchema, Serialize)]
pub struct Files {
    /// A mutual exclusion lock used by a watcher thread
    /// when updating this file set
    #[serde(flatten)]
    pub registry: FileRegistry,

    #[serde(skip)]
    pub watcher: Option<std::sync::mpsc::Sender<()>>,
}

type FileRegistry = Arc<Mutex<BTreeMap<PathBuf, File>>>;

impl Files {
    /// Load files from a folder
    pub fn load(folder: &str, watch: bool) -> Result<Files> {
        let path = Path::new(folder).canonicalize()?;

        // Collect all the files
        let mut files = walkdir::WalkDir::new(&path)
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

        // Resolve `parent` and `children` properties
        // This needs to clone the files to avoid mutable borrow twice,
        // there may be a more efficient alternative
        let mut children: BTreeMap<PathBuf, BTreeSet<PathBuf>> = BTreeMap::new();
        for (path, file) in &mut files {
            if let Some(parent_path) = path.parent() {
                let parent_path = parent_path.to_path_buf();
                file.parent = Some(parent_path.clone());
                children
                    .entry(parent_path)
                    .or_insert_with(BTreeSet::new)
                    .insert(path.clone());
            }
        }
        for (path, vec) in children {
            if let Some(parent) = files.get_mut(&path) {
                parent.children = Some(vec)
            }
        }

        // Create a registry of the files
        let registry = Arc::new(Mutex::new(files));

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

                let handle_event = |event| match event {
                    DebouncedEvent::Create(path) => Files::created(&project, &registry, &path),
                    DebouncedEvent::Remove(path) => Files::removed(&project, &registry, &path),
                    DebouncedEvent::Rename(from, to) => {
                        Files::renamed(&project, &registry, &from, &to)
                    }
                    DebouncedEvent::Write(path) => Files::modified(&project, &registry, &path),
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
    pub fn obtain(&self) -> Result<MutexGuard<BTreeMap<PathBuf, File>>> {
        match self.registry.try_lock() {
            Ok(registry) => Ok(registry),
            Err(error) => bail!("Unable to get file registry: {}", error),
        }
    }

    // Update a project file registry when a file is created
    pub fn created(project: &Path, registry: &FileRegistry, path: &Path) {
        let mut registry = registry.lock().unwrap();

        // Load the file, insert it and add it to it's parent's children
        let file = if let Ok((path, file)) = File::load(path) {
            registry.insert(path.clone(), file.clone());
            if let Some(parent) = path.parent().and_then(|parent| registry.get_mut(parent)) {
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
            files: Some(registry.clone()),
        })
    }

    // Update a project file registry when a file is removed
    pub fn removed(project: &Path, registry: &FileRegistry, path: &Path) {
        let mut registry = registry.lock().unwrap();

        // Remove the file and remove it from its parent's children
        registry.remove(path);
        if let Some(parent) = path.parent().and_then(|parent| registry.get_mut(parent)) {
            if let Some(children) = &mut parent.children {
                children.remove(path);
            }
        }

        pubsub::publish_project_file(ProjectFileEvent {
            project: project.into(),
            path: path.into(),
            kind: "removed".into(),
            file: None,
            files: Some(registry.clone()),
        })
    }

    // Update a project file registry when a file is renamed
    pub fn renamed(project: &Path, registry: &FileRegistry, old_path: &Path, new_path: &Path) {
        let mut registry = registry.lock().unwrap();

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

        // Move the file
        let file = match registry.entry(old_path.into()) {
            Entry::Occupied(entry) => {
                // Update it's path and parent properties, and the paths of
                // all it's descendants. Move the file from old to new path.
                let mut file = entry.remove();
                file.path = new_path.into();
                file.parent = new_path.parent().map(|parent| parent.into());
                rename_children(&mut *registry, &mut file, old_path, new_path);
                registry.insert(new_path.into(), file.clone());
                Some(file)
            }
            Entry::Vacant(_) => {
                // The entry should not be empty, but in case it is, load the file from,
                // and insert it at, the new path
                if let Ok((_path, file)) = File::load(new_path) {
                    registry.insert(new_path.into(), file.clone());
                    Some(file)
                } else {
                    None
                }
            }
        };

        // Remove the old path from the old parent's children
        if let Some(parent) = old_path
            .parent()
            .and_then(|parent| registry.get_mut(parent))
        {
            if let Some(children) = &mut parent.children {
                children.remove(old_path);
            }
        }

        // Insert the new path to the new parent's children
        if let Some(parent) = new_path
            .parent()
            .and_then(|parent| registry.get_mut(parent))
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
            files: Some(registry.clone()),
        })
    }

    // Update a project file registry when a file is modified
    pub fn modified(project: &Path, registry: &FileRegistry, path: &Path) {
        let mut registry = registry.lock().unwrap();

        // Insert the file
        let file = if let Ok((path, file)) = File::load(path) {
            registry.insert(path, file.clone());
            Some(file)
        } else {
            None
        };

        pubsub::publish_project_file(ProjectFileEvent {
            project: project.into(),
            path: path.into(),
            kind: "modified".into(),
            file,
            files: Some(registry.clone()),
        })
    }
}
