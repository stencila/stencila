use crate::utils::schemas;
use defaults::Defaults;
use events::publish;
use formats::FormatSpec;
use schemars::{schema::Schema, JsonSchema};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use strum::Display;

/// A file or directory within a `Project`
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
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
    #[def = "FormatSpec::unknown(\"unknown\")"]
    #[schemars(schema_with = "File::schema_format")]
    pub format: FormatSpec,

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
    /// Generate the JSON Schema for the `format` property to avoid nested type.
    fn schema_format(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("Format", true)
    }

    /// Get a file's name from it's path
    pub fn name(path: &Path) -> String {
        path.file_name()
            .map(|os_str| os_str.to_string_lossy())
            .unwrap_or_default()
            .into()
    }

    /// Get a file's parent from it's path
    pub fn parent(path: &Path) -> Option<PathBuf> {
        path.parent().map(|parent| parent.into())
    }

    /// Load a file from a path
    ///
    /// Note: this function is infallible, in that it will always return a
    /// `File`. However, if there were errors obtaining a field it will be
    /// `None`, or possible erroneous (e.g. in the unlikely event that
    /// `path.canonicalize()` fails for example). Having this function return
    /// a `File`, instead of a `Result<File>` simplifies other code substantially.
    pub fn load(path: &Path) -> File {
        let path = path.canonicalize().unwrap_or_else(|_error| path.into());

        let name = File::name(&path);
        let parent = File::parent(&path);

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

        let (format, children) = if path.is_file() {
            (formats::match_path(&path).spec(), None)
        } else {
            (FormatSpec::directory(), Some(BTreeSet::new()))
        };

        File {
            path,
            name,
            modified,
            size,
            format,
            parent,
            children,
        }
    }
}

#[derive(Display, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileEventType {
    Refreshed,
    Created,
    Removed,
    Renamed,
    Modified,
}

/// An event associated with a `File` or a set of `File`s
///
/// These events published under the `projects:<project-path>:files` topic.
#[derive(JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct FileEvent {
    /// The path of the project (absolute)
    pub project: PathBuf,

    /// The path of the file (absolute)
    ///
    /// For `renamed` events this is the _old_ path.
    pub path: PathBuf,

    /// The type of event e.g. `Refreshed`, `Modified`, `Created`
    ///
    /// A `refreshed` event is emitted when the entire set of
    /// files is updated.
    #[serde(rename = "type")]
    pub type_: FileEventType,

    /// The updated file
    ///
    /// Will be `None` for for `refreshed` and `removed` events,
    /// or if for some reason it was not possible to fetch metadata
    /// about the file.
    #[schemars(schema_with = "FileEvent::schema_file")]
    pub file: Option<File>,

    /// The updated set of files in the project
    ///
    /// Represents the new state of the file tree after the
    /// event including updated `parent` and `children`
    /// properties of files affects by the event.
    #[schemars(schema_with = "FileEvent::schema_files")]
    pub files: BTreeMap<PathBuf, File>,
}

impl FileEvent {
    /// Generate the JSON Schema for the `file` property
    fn schema_file(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("File", false)
    }

    /// Generate the JSON Schema for the `files` property
    fn schema_files(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("Record<string, File>", true)
    }

    pub fn publish(
        project: &Path,
        path: &Path,
        type_: FileEventType,
        file: Option<File>,
        files: &BTreeMap<PathBuf, File>,
    ) {
        let topic = &format!(
            "projects:{}:files:{}:{}",
            project.display(),
            path.display(),
            type_
        );
        let event = FileEvent {
            project: project.into(),
            path: path.into(),
            type_,
            file,
            files: files.clone(),
        };
        publish(topic, &event)
    }
}

/// A registry of `File`s within a `Project`
#[derive(Clone, Debug, Default, JsonSchema, Serialize)]
pub struct Files {
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

impl Files {
    const GITIGNORE_NAMES: [&'static str; 2] = [".ignore", ".gitignore"];

    pub fn new<P: AsRef<Path>>(path: P) -> Files {
        let (files, ignore_files) = Files::walk(&path);
        Files {
            path: path.as_ref().to_path_buf(),
            files,
            ignore_files,
            ..Default::default()
        }
    }

    /// Walk a path and collect file and Git ignore files from it
    pub fn walk<P: AsRef<Path>>(path: P) -> (BTreeMap<PathBuf, File>, BTreeSet<PathBuf>) {
        // Build walker
        let walker = ignore::WalkBuilder::new(&path)
            // Respect .ignore files
            .ignore(true)
            // Respect .gitignore files
            .git_ignore(true)
            .build_parallel();

        // Collect files in parallel using a collector thread and several walker thread
        // (number of which is chosen by the `ignore` walker)
        let (sender, receiver) = crossbeam_channel::bounded(100);
        let join_handle =
            std::thread::spawn(move || -> BTreeMap<PathBuf, File> { receiver.iter().collect() });
        walker.run(|| {
            let sender = sender.clone();
            Box::new(move |result| {
                use ignore::WalkState::*;

                if let Ok(entry) = result {
                    let path = entry.path();
                    let file = File::load(path);
                    sender
                        .send((file.path.clone(), file))
                        .expect("Unable to send to collector");
                }

                Continue
            })
        });
        drop(sender);
        let mut files = join_handle.join().expect("Unable to join collector thread");

        // Resolve `children` properties and `ignore_files` files
        let mut ignore_files = BTreeSet::new();
        for path in files.keys().cloned().collect::<Vec<PathBuf>>() {
            if Files::is_ignore_file(&path) {
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

        (files, ignore_files)
    }

    /// Should the registry be refreshed in response to a change in a file
    ///
    /// For example if a `.gitignore` file is added, removed, moved or modified.
    fn should_refresh(&mut self, path: &Path) -> bool {
        Files::is_ignore_file(path)
    }

    /// Refresh the registry if it should be
    fn did_refresh(&mut self, path: &Path) -> bool {
        if self.should_refresh(path) {
            self.refresh();
            true
        } else {
            false
        }
    }

    /// Is the file a Git ignore file?
    fn is_ignore_file(path: &Path) -> bool {
        let name = File::name(path);
        Files::GITIGNORE_NAMES.contains(&name.as_str())
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
                    if let Ok(ignore_file) = gitignore::File::new(ignore_file_path) {
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

    /// Get the parent `File` of a path, ensure that all it's
    /// ancestors exist, and add the path as a child.
    ///
    /// This is used to ensure that the ancestor `File`s of a path exists
    /// in the registry (e.g. when a new file is created or renamed in a sub folder)
    /// and that the current path is added as a child.
    /// It will return `None` if the path has no parent (i.e is outside of the root)
    fn ensure_ancestors(&mut self, path: &Path) -> Option<&mut File> {
        if let Some(parent) = path.parent() {
            if !parent.starts_with(&self.path) {
                return None;
            }

            self.ensure_ancestors(parent);

            let parent = self
                .files
                .entry(parent.into())
                .or_insert_with(|| File::load(parent));

            if let Some(children) = &mut parent.children {
                children.insert(path.into());
            }

            Some(parent)
        } else {
            None
        }
    }

    /// Refresh the file registry
    fn refresh(&mut self) {
        *self = Files::new(self.path.as_path());

        FileEvent::publish(
            &self.path,
            Path::new("*"),
            FileEventType::Refreshed,
            None,
            &self.files,
        )
    }

    // Update the file registry when a file is created
    pub fn created(&mut self, path: &Path) {
        if self.should_ignore(path) || self.did_refresh(path) {
            return;
        }

        // Load the file, insert it and add it to it's parent's children
        let file = File::load(path);
        self.files.insert(path.into(), file.clone());
        self.ensure_ancestors(path);

        if path.is_dir() {
            // If the path created is a directory with empty sub-directories
            // we only get an event for the top level.
            // e.g. for `mkdir -p a/b/c` we only get an event for `a` being created.
            // So we have to walk it. This is potentially wasteful because we may
            // already loaded files when getting individual file `created` events
            // or when walking subdirectories (e.g. when a zip file is extracted).
            // But there does not seem to be an easy, safe alternative.
            let (files, ignore_files) = &mut Files::walk(path);
            self.files.append(files);
            self.ignore_files.append(ignore_files);
        } else {
            // If it's a file, we may need to add it to the ignore files
            if Files::is_ignore_file(path) {
                self.ignore_files.insert(path.into());
            }
        }

        FileEvent::publish(
            &self.path,
            path,
            FileEventType::Created,
            Some(file),
            &self.files,
        )
    }

    // Update the file registry when a file is removed
    pub fn removed(&mut self, path: &Path) {
        if self.should_ignore(path) || self.did_refresh(path) {
            return;
        }

        // Remove the file and remove it from its parent's children
        self.files.remove(path);
        if let Some(parent) = self.files.get_mut(path) {
            if let Some(children) = &mut parent.children {
                children.remove(path);
            }
        }

        FileEvent::publish(&self.path, path, FileEventType::Removed, None, &self.files)
    }

    // Update the file registry when a file is renamed
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
                // Update it's path, name etc, and the paths of
                // all it's descendants. Move the file from old to new path.
                let mut file = entry.remove();
                file.path = new_path.into();
                file.name = File::name(new_path);
                file.parent = File::parent(new_path);
                file.format = formats::match_path(&new_path).spec();
                rename_children(&mut self.files, &mut file, old_path, new_path);
                self.files.insert(new_path.into(), file.clone());
                file
            }
            Entry::Vacant(_) => {
                // The entry should not be empty, but in case it is, load the file from,
                // and insert it at, the new path
                let file = File::load(new_path);
                self.files.insert(new_path.into(), file.clone());
                file
            }
        };

        // Recursively rename children of a `File` (if it has `children` i.e. is a directory)
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

        // Remove the old path from the old path's parent's children
        if let Some(parent_path) = old_path.parent() {
            if let Some(parent) = self.files.get_mut(parent_path) {
                if let Some(children) = &mut parent.children {
                    children.remove(old_path);
                }
            }
        }

        // Insert the new path to the new parent's children
        self.ensure_ancestors(new_path);

        FileEvent::publish(
            &self.path,
            old_path,
            FileEventType::Renamed,
            Some(file),
            &self.files,
        )
    }

    // Update the file registry when a file is modified
    pub fn modified(&mut self, path: &Path) {
        if self.should_ignore(path) || self.did_refresh(path) {
            return;
        }

        // Insert the file
        let file = File::load(path);
        self.files.insert(path.into(), file.clone());

        FileEvent::publish(
            &self.path,
            path,
            FileEventType::Modified,
            Some(file),
            &self.files,
        )
    }
}
