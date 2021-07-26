use crate::files::{File, FileEvent, Files};
use crate::methods::import::import;
use crate::pubsub::publish;
use crate::sources::{self, Source, SourceDestination, SourceTrait};
use crate::utils::schemas;
use eyre::{bail, Result};
use notify::DebouncedEvent;
use regex::Regex;
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use slug::slugify;
use std::string::ToString;
use std::sync::Arc;
use std::time::Duration;
use std::{
    collections::{hash_map::Entry, HashMap},
    fs,
    path::{Path, PathBuf},
};
use strum::ToString;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Debug, JsonSchema, Serialize, ToString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
enum ProjectEventType {
    Updated,
}

#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct ProjectEvent {
    /// The project associated with the event
    #[schemars(schema_with = "ProjectEvent::schema_project")]
    project: Project,

    /// The type of event
    #[serde(rename = "type")]
    type_: ProjectEventType,
}

impl ProjectEvent {
    /// Generate the JSON Schema for the `project` property to avoid nesting
    fn schema_project(_generator: &mut SchemaGenerator) -> Schema {
        schemas::typescript("Project", true)
    }

    /// Publish a `ProjectEvent`.
    ///
    /// Will publish the events under the `projects:<>:props` topic
    /// so it can be differentiated from `FileEvents` under the
    /// `projects:{}:files` topic.
    pub fn publish(project: &Project, type_: ProjectEventType) {
        let topic = &format!("projects:{}:props", project.path.display());
        let event = ProjectEvent {
            project: project.clone(),
            type_,
        };
        publish(topic, &event)
    }
}

/// Details of a project
///
/// An implementation, and extension, of schema.org [`Project`](https://schema.org/Project).
/// Uses schema.org properties where possible but adds extension properties
/// where needed (e.g. `theme`).
#[skip_serializing_none]
#[derive(Clone, Debug, Default, JsonSchema, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
#[schemars(deny_unknown_fields)]
pub struct Project {
    /// The name of the project
    name: Option<String>,

    /// A description of the project
    description: Option<String>,

    /// The path (within the project) of the project's image
    ///
    /// If not specified, will default to the most recently
    /// modified image in the project (if any).
    image: Option<String>,

    /// The path (within the project) of the project's main file
    ///
    /// If not specified, will default to the first file matching the
    /// the regular expression in the configuration settings.
    main: Option<String>,

    /// The default theme to use when viewing documents in this project
    ///
    /// If not specified, will default to the default theme in the
    /// configuration settings.
    theme: Option<String>,

    /// A list of project sources and their destination within the project
    #[schemars(schema_with = "Project::schema_sources")]
    pub sources: Option<HashMap<String, SourceDestination>>,

    /// Glob patterns for paths to be excluded from file watching
    ///
    /// As a performance optimization, paths that match these patterns are
    /// excluded from file watching updates.
    /// If not specified, will default to the patterns in the
    /// configuration settings.
    watch_exclude_patterns: Option<Vec<String>>,

    // The following properties are derived from the filesystem
    // and should never be read from, or written to, the `project.json` file.
    // The `write` method excludes them writing.
    /// The filesystem path of the project folder
    #[serde(skip_deserializing)]
    #[schemars(schema_with = "Project::schema_path")]
    path: PathBuf,

    /// The resolved path of the project's image file
    #[serde(skip_deserializing)]
    image_path: Option<PathBuf>,

    /// The resolved path of the project's main file
    #[serde(skip_deserializing)]
    pub main_path: Option<PathBuf>,

    /// The files in the project folder
    #[serde(skip_deserializing)]
    #[schemars(schema_with = "Project::schema_files")]
    files: Files,
}

impl Project {
    /// Generate the JSON Schema for the `path` property to avoid optionality
    /// due to `skip_deserializing`
    fn schema_path(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("string", true)
    }

    /// Generate the JSON Schema for the `source` property to avoid duplicated types
    fn schema_sources(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("SourceDestination[]", false)
    }

    /// Generate the JSON Schema for the `file` property to avoid duplicated
    /// inline type and optionality due to `skip_deserializing`
    fn schema_files(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("Record<string, File>", true)
    }

    /// The name of the project's manifest file within the project directory
    const FILE_NAME: &'static str = "project.json";

    /// Get the path to a project's manifest file
    fn file<P: AsRef<Path>>(path: P) -> PathBuf {
        path.as_ref().join(Project::FILE_NAME)
    }

    /// Load a project's from its manifest file
    ///
    /// If there is no manifest file, then default values will be used
    fn load<P: AsRef<Path>>(path: P) -> Result<Project> {
        let path = path.as_ref();
        if !path.exists() {
            bail!("Project folder does not exist: {}", path.display())
        }
        let path = path.canonicalize()?;

        let file = Project::file(&path);
        let mut project = if file.exists() {
            let json = fs::read_to_string(file)?;
            serde_json::from_str(&json)?
        } else {
            Project::default()
        };
        project.path = path;

        Ok(project)
    }

    /// Open a project from an existing folder
    ///
    /// Reads the `project.json` file (if any) and walks the project folder
    /// to build a filesystem tree.
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Project> {
        // Load the project manifest (if any).
        let mut project = Project::load(&path)?;

        // Get all the files and folders in the project
        project.files = Files::new(&path);

        // Update the project's properties,
        // some one which may depend on the files.
        project.update(None).await;

        Ok(project)
    }

    /// Initialize a project in a new, or existing, folder
    ///
    /// If the project has already been initialized (i.e. has a manifest file)
    /// then this function will do nothing.
    pub async fn init<P: AsRef<Path>>(path: P) -> Result<()> {
        if Project::file(&path).exists() {
            return Ok(());
        }

        if !path.as_ref().exists() {
            fs::create_dir_all(&path)?;
        }

        let mut project = Project::open(path).await?;
        project.write(None).await?;

        Ok(())
    }

    /// Read a project's manifest file and update the project
    ///
    /// Overwrites properties with values from the file and then
    /// calls `update()`. If there is no manifest file, then default
    /// values will be used.
    async fn read(&mut self) -> Result<()> {
        let file = Project::file(&self.path);
        let updates = if file.exists() {
            let json = fs::read_to_string(file)?;
            serde_json::from_str(&json)?
        } else {
            Project::default()
        };

        self.update(Some(updates)).await;

        Ok(())
    }

    /// Write a project's manifest file, optionally providing updated properties
    ///
    /// If the project folder does not exist yet then it will be created.
    pub async fn write(&mut self, updates: Option<Project>) -> Result<()> {
        if updates.is_some() {
            self.update(updates).await
        }

        // Redact derived properties
        let mut value = serde_json::to_value(&self)?;
        let map = value.as_object_mut().expect("Should always be an object");
        map.remove("path");
        map.remove("imagePath");
        map.remove("mainPath");
        map.remove("files");
        let json = serde_json::to_string_pretty(map)?;

        let path = &self.path;
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        let file = Project::file(path);
        fs::write(file, json)?;

        Ok(())
    }

    /// Update a project after changes to it's package.json or main file.
    ///
    /// Attempts to use the projects `main` property. If that is not specified, or
    /// there is no matching file in the project, attempts to match one of the
    /// project's files against the `main_patterns`.
    async fn update(&mut self, updates: Option<Project>) {
        tracing::debug!("Updating project: {}", self.path.display());

        if let Some(updates) = updates {
            self.name = updates.name;
            self.description = updates.description;
            self.main = updates.main;
            self.image = updates.image;
            self.theme = updates.theme;
            self.watch_exclude_patterns = updates.watch_exclude_patterns;
        }

        let files = &self.files.files;

        let config = &crate::config::lock().await.projects;

        // Resolve the main file path first as some of the other project properties
        // may be defined there (e.g. in the YAML header of a Markdown file)
        let main_path = (|| {
            // Check that there is a file with the specified main path
            if let Some(main) = &self.main {
                let main_path = self.path.join(main);
                if files.contains_key(&main_path) {
                    return Some(main_path);
                } else {
                    tracing::warn!("Project main file specified could not be found: {}", main);
                    // Will attempt to find using patterns
                }
            }

            // For each `main_pattern` (in order)...
            for pattern in &config.main_patterns {
                // Make matching case insensitive
                let re = match Regex::new(&pattern.to_lowercase()) {
                    Ok(re) => re,
                    Err(_) => {
                        tracing::warn!("Project main file pattern is invalid: {}", pattern);
                        continue;
                    }
                };

                for file in files.values() {
                    // Ignore directories
                    if file.children.is_some() {
                        continue;
                    }
                    // Match relative path to pattern
                    if let Ok(relative_path) = file.path.strip_prefix(&self.path) {
                        if re.is_match(&relative_path.to_string_lossy().to_lowercase()) {
                            return Some(file.path.clone());
                        }
                    }
                }
            }

            None
        })();

        // Canonicalize the main file path so, amongst other things, it can be matched to
        // project documents. If the file does not exist, will be none
        self.main_path = main_path.map(|path| path.canonicalize().ok().unwrap_or_default());

        // Name defaults to the name of the project's folder
        self.name = self
            .name
            .clone()
            .or_else(|| match &self.path.components().last() {
                Some(last) => Some(last.as_os_str().to_string_lossy().to_string()),
                None => Some("Unnamed".to_string()),
            });

        // Theme defaults to the configured default
        self.theme = self.theme.clone().or_else(|| Some(config.theme.clone()));

        ProjectEvent::publish(self, ProjectEventType::Updated)
    }

    /// Show a project
    ///
    /// Generates a Markdown representation of a project.
    /// Used for by `stencila projects show` and possibly elsewhere.
    pub fn show(&self) -> Result<String> {
        use handlebars::Handlebars;

        let template = r#"
# {{name}}

**Project path**: {{ path }}
**Main file**: {{ mainPath }}
**Image file**: {{ imagePath }}
**Theme**: {{ theme }}

"#;
        let hb = Handlebars::new();
        let md = hb.render_template(template.trim(), self)?;
        Ok(md)
    }

    /// Add a source to the project
    pub async fn add_source(
        &mut self,
        source: &str,
        destination: Option<String>,
        name: Option<String>,
    ) -> Result<Source> {
        // Resolve the source
        let source = sources::resolve(source)?;

        // Ensure that the name is unique
        let name = if let Some(name) = name {
            if let Some(sources) = &self.sources {
                if sources.contains_key(&name) {
                    bail!("A source with with name already exists; please use a different name")
                }
            }
            name
        } else {
            let mut name = source.default_name();
            if let Some(sources) = &self.sources {
                if sources.contains_key(&name) {
                    if let Some(destination) = destination.as_ref() {
                        name = [name, "-".to_string(), slugify(destination)].concat();
                    }
                }
                while sources.contains_key(&name) {
                    name += "-"
                }
            }
            name
        };

        // Add the source (if not already a source in the project)
        if let Some(sources) = &mut self.sources {
            for (name, source_dest) in sources.iter() {
                if source == source_dest.source && destination == source_dest.destination {
                    bail!(
                        "The source/destination combination already exists ('{}'); perhaps remove it or use a different destination?",
                        name
                    )
                }
            }
            sources.insert(
                name,
                SourceDestination {
                    source: source.clone(),
                    destination,
                },
            );
        } else {
            let mut sources = HashMap::new();
            sources.insert(
                name,
                SourceDestination {
                    source: source.clone(),
                    destination,
                },
            );
            self.sources = Some(sources)
        };
        self.write(None).await?;

        Ok(source)
    }

    /// Remove a source from the project
    pub async fn remove_source(&mut self, name: &str) -> Result<()> {
        if let Some(sources) = &mut self.sources {
            let len = sources.len();
            sources.remove(name);
            if sources.len() == len {
                tracing::warn!("Project has no sources with name '{}'", name)
            } else {
                self.write(None).await?;
            }
        } else {
            tracing::warn!("Project has no sources")
        }

        Ok(())
    }

    /// Import a source into the project
    pub async fn import_source(
        &mut self,
        name_or_identifier: &str,
        destination: Option<String>,
    ) -> Result<Vec<File>> {
        // Attempt to find a source with matching name
        let mut source = if let Some(sources) = &self.sources {
            sources
                .get(name_or_identifier)
                .map(|source_dest| source_dest.source.clone())
        } else {
            None
        };

        // Attempt to find an existing entry with matching source/destination combination
        if source.is_none() {
            if let Some(sources) = &self.sources {
                let source_from = sources::resolve(name_or_identifier)?;
                for source_dest in sources.values() {
                    if source_dest.source == source_from || source_dest.destination == destination {
                        source = Some(source_from);
                        break;
                    }
                }
            }
        }

        // Add the source if necessary
        let source = if let Some(source) = source {
            source
        } else {
            self.add_source(name_or_identifier, destination.clone(), None)
                .await?
        };

        // Import the source
        let files = import(&self.path, &source, destination).await?;

        Ok(files)
    }
}

#[derive(Debug)]
pub struct ProjectHandler {
    /// The project being handled.
    project: Arc<Mutex<Project>>,

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

impl Drop for ProjectHandler {
    fn drop(&mut self) {
        match &self.handler {
            Some(handler) => handler.abort(),
            None => {}
        }
    }
}

impl ProjectHandler {
    /// Open a project and, optionally, watch it for changes
    pub async fn open<P: AsRef<Path>>(folder: P, watch: bool) -> Result<ProjectHandler> {
        let project = Project::open(&folder).await?;
        let handler = ProjectHandler::new(project, watch).await;
        Ok(handler)
    }

    /// Create a new project handler.
    ///
    /// # Arguments
    ///
    /// - `project`: The project that this handler is for.
    /// - `watch`: Whether to watch the project (e.g. not for temporary, new files)
    async fn new(project: Project, watch: bool) -> ProjectHandler {
        let path = project.path.clone();

        // Watch exclude patterns default to the configured defaults.
        // Note that this project setting can not be updated while it is being watched.
        let config = &crate::config::lock().await.projects;
        let watch_exclude_patterns = project
            .watch_exclude_patterns
            .clone()
            .unwrap_or_else(|| config.watch_exclude_patterns.clone());

        let project = Arc::new(Mutex::new(project));

        let (watcher, handler) = if watch {
            let (watcher, handler) =
                ProjectHandler::watch(path, watch_exclude_patterns, Arc::clone(&project));
            (Some(watcher), Some(handler))
        } else {
            (None, None)
        };

        ProjectHandler {
            project,
            watcher,
            handler,
        }
    }

    const WATCHER_DELAY_MILLIS: u64 = 300;

    /// Watch the project.
    fn watch(
        path: PathBuf,
        watch_exclude_patterns: Vec<String>,
        project: Arc<Mutex<Project>>,
    ) -> (crossbeam_channel::Sender<()>, JoinHandle<()>) {
        let (thread_sender, thread_receiver) = crossbeam_channel::bounded(1);
        let (async_sender, mut async_receiver) = tokio::sync::mpsc::channel(100);

        let path_clone = path.clone();

        // Standard thread to run blocking sync file watcher
        std::thread::spawn(move || -> Result<()> {
            use notify::{watcher, RecursiveMode, Watcher};

            let (watcher_sender, watcher_receiver) = std::sync::mpsc::channel();
            let mut watcher = watcher(
                watcher_sender,
                Duration::from_millis(ProjectHandler::WATCHER_DELAY_MILLIS),
            )?;
            watcher.watch(&path, RecursiveMode::Recursive)?;

            // Event checking timeout. Can be quite long since only want to check
            // whether we can end this thread.
            let timeout = Duration::from_millis(100);

            let path_string = path.display().to_string();
            let span = tracing::info_span!("project_watcher", project = path_string.as_str());
            let _enter = span.enter();
            tracing::debug!("Starting project watcher: {}", path_string);
            loop {
                // Check for an event. Use `recv_timeout` so we don't
                // get stuck here and will do following check that ends this
                // thread if the owning `DocumentHandler` is dropped
                if let Ok(event) = watcher_receiver.recv_timeout(timeout) {
                    tracing::debug!("Event for project '{}': {:?}", path_string, event);
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
            tracing::debug!("Ending project watcher: {}", path_string);

            // Drop the sync send so that the event handling thread also ends
            drop(async_sender);

            Ok(())
        });

        // Async task to handle events
        let handler = tokio::spawn(async move {
            // The globs of files to be excluded from the watch
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

            // Should the event trigger an update to the project files?
            let should_update_files = |event_path: &Path| {
                if let Ok(event_path) = event_path.strip_prefix(&path_clone) {
                    for glob in &exclude_globs {
                        if glob.matches(&event_path.display().to_string()) {
                            return false;
                        }
                    }
                }
                true
            };

            // Should the event trigger an update to other project properties?
            let should_read_project = |event_path: &Path| {
                if let Some(file_name) = event_path.file_name() {
                    if file_name == Project::FILE_NAME {
                        return true;
                    }
                }
                false
            };

            // Read the project
            async fn read_project(project: &mut Project) {
                if let Err(error) = project.read().await {
                    tracing::error!(
                        "While reading project '{}': {}",
                        project.path.display(),
                        error
                    )
                }
            }

            tracing::debug!("Starting project handler");
            while let Some(event) = async_receiver.recv().await {
                match event {
                    DebouncedEvent::Create(path) => {
                        let project = &mut *project.lock().await;
                        if should_update_files(&path) {
                            project.files.created(&path);
                        }
                        if should_read_project(&path) {
                            read_project(project).await;
                        }
                    }
                    DebouncedEvent::Remove(path) => {
                        let project = &mut *project.lock().await;
                        if should_update_files(&path) {
                            project.files.removed(&path);
                        }
                        if should_read_project(&path) {
                            read_project(project).await;
                        }
                    }
                    DebouncedEvent::Rename(from, to) => {
                        let project = &mut *project.lock().await;
                        if should_update_files(&from) || should_update_files(&to) {
                            project.files.renamed(&from, &to);
                        }
                        if should_read_project(&from) || should_read_project(&to) {
                            read_project(project).await;
                        }
                    }
                    DebouncedEvent::Write(path) => {
                        let project = &mut *project.lock().await;
                        if should_update_files(&path) {
                            project.files.modified(&path);
                        }
                        if should_read_project(&path) {
                            read_project(project).await;
                        }
                    }
                    _ => {}
                }
            }
            // Because we abort this thread, this entry may never get
            // printed (only if the `async_sender` is dropped before this is aborted)
            tracing::debug!("Ending project handler");
        });

        (thread_sender, handler)
    }
}

/// An in-memory store of projects and associated
/// data (e.g. file system watchers)
#[derive(Debug, Default)]
pub struct Projects {
    /// The projects, stored by absolute path
    pub registry: HashMap<PathBuf, ProjectHandler>,
}

impl Projects {
    /// Create a new project
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the path of the current project, falling back to the current working directory
    ///
    /// Searches up the directory tree for a `project.json`, returning the parent of the
    /// first one found. If none is found returns the current working directory.
    pub fn current_path() -> Result<PathBuf> {
        let current = std::env::current_dir()?;
        let mut dir = current.clone();
        loop {
            if dir.join(Project::FILE_NAME).exists() {
                return Ok(dir);
            }
            match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => return Ok(current),
            }
        }
    }

    /// List documents that are currently open
    ///
    /// Returns a vector of document paths (relative to the current working directory)
    pub async fn list(&self) -> Result<Vec<String>> {
        let cwd = std::env::current_dir()?;
        let mut paths = Vec::new();
        for project in self.registry.values() {
            let path = &project.project.lock().await.path;
            let path = match pathdiff::diff_paths(path, &cwd) {
                Some(path) => path,
                None => path.clone(),
            };
            let path = path.display().to_string();
            paths.push(path);
        }
        Ok(paths)
    }

    /// Open a project
    pub async fn open<P: AsRef<Path>>(&mut self, path: Option<P>, watch: bool) -> Result<Project> {
        let path = match path {
            Some(path) => path.as_ref().canonicalize()?,
            None => Projects::current_path()?,
        };

        let handler = match self.registry.entry(path.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => match ProjectHandler::open(path, watch).await {
                Ok(handler) => entry.insert(handler),
                Err(error) => return Err(error),
            },
        };

        Ok(handler.project.lock().await.clone())
    }

    /// Open the current project
    pub async fn current(&mut self, watch: bool) -> Result<Project> {
        self.open::<PathBuf>(None, watch).await
    }

    /// Get a project
    pub fn get<P: AsRef<Path>>(&mut self, path: P) -> Result<Arc<Mutex<Project>>> {
        let path = path.as_ref().canonicalize()?;

        if let Some(handler) = self.registry.get(&path) {
            Ok(handler.project.clone())
        } else {
            bail!("No project with path {}", path.display())
        }
    }

    /// Write a project
    pub async fn write<P: AsRef<Path>>(&mut self, path: P, updates: Option<Project>) -> Result<()> {
        self.get(&path)?.lock().await.write(updates).await
    }

    /// Close a project
    pub fn close<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().canonicalize()?;
        self.registry.remove(&path);
        Ok(())
    }
}

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![
        schemas::generate::<Project>()?,
        schemas::generate::<ProjectEvent>()?,
        schemas::generate::<File>()?,
        schemas::generate::<FileEvent>()?,
    ]);
    Ok(schemas)
}

pub mod config {
    use super::*;
    use defaults::Defaults;
    use validator::Validate;

    /// Projects
    ///
    /// Configuration settings for project defaults
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default, rename_all = "camelCase")]
    #[schemars(deny_unknown_fields)]
    pub struct ProjectsConfig {
        /// Patterns used to infer the main file of projects
        ///
        /// For projects that do not specify a main file, each file is tested
        /// against these case insensitive patterns in order. The first
        /// file (alphabetically) that matches is the project's main file.
        #[def = r#"vec!["^main\\b".to_string(), "^index\\b".to_string(), "^readme\\b".to_string()]"#]
        pub main_patterns: Vec<String>,

        /// Default project theme
        ///
        /// Will be applied to all projects that do not specify a theme
        #[def = r#"String::from("stencila")"#]
        pub theme: String,

        /// Default glob patterns for paths to be excluded from file watching
        ///
        /// Used for projects that do not specify their own watch exclude patterns.
        /// As a performance optimization, paths that match these patterns are
        /// excluded from file watching updates.
        /// The default list includes common directories that often have many files
        /// that are often updated.
        #[def = r#"vec!["*/.git".to_string(), "^*/node_modules".to_string()]"#]
        pub watch_exclude_patterns: Vec<String>,
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use crate::cli::display;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage projects",
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
        Init(Init),
        List(List),
        Open(Open),
        Close(Close),
        Show(Show),
        Schemas(Schemas),
    }

    impl Command {
        pub async fn run(self, projects: &mut Projects) -> display::Result {
            let Self { action } = self;
            match action {
                Action::Init(action) => action.run().await,
                Action::List(action) => action.run(projects).await,
                Action::Open(action) => action.run(projects).await,
                Action::Close(action) => action.run(projects),
                Action::Show(action) => action.run(projects).await,
                Action::Schemas(action) => action.run(),
            }
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Initialize a project in a new, or existing, folder",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Init {
        /// The path of the new, or existing, folder to initialize
        ///
        /// If no folder exists at the path, then one will be created.
        /// If no `project.json` file exists in the folder then a new one
        /// will be created.
        #[structopt(default_value = ".")]
        pub folder: PathBuf,
    }

    impl Init {
        pub async fn run(&self) -> display::Result {
            let Self { folder } = self;
            Project::init(folder).await?;
            display::nothing()
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "List open projects",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}

    impl List {
        pub async fn run(&self, projects: &mut Projects) -> display::Result {
            let list = projects.list().await?;
            display::value(list)
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Open a project",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Open {
        /// The path of the project folder (defaults to the current project)
        pub folder: Option<PathBuf>,
    }

    impl Open {
        pub async fn run(self, projects: &mut Projects) -> display::Result {
            let Self { folder } = self;
            let Project { name, .. } = projects.open(folder, true).await?;
            display::value(name)
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Close a project",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Close {
        /// The path of the project folder
        #[structopt(default_value = ".")]
        pub folder: PathBuf,
    }

    impl Close {
        pub fn run(&self, projects: &mut Projects) -> display::Result {
            let Self { folder } = self;
            projects.close(folder)?;
            display::nothing()
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Show a project details",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// The path of the project folder (defaults to the current project)
        pub folder: Option<PathBuf>,
    }

    impl Show {
        pub async fn run(self, projects: &mut Projects) -> display::Result {
            let Self { folder } = self;
            let project = projects.open(folder, false).await?;
            let content = project.show()?;
            display::new("md", &content, Some(project))
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Get JSON schemas for projects and associated types",
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
