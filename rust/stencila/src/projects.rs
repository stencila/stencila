use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use notify::DebouncedEvent;
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};

use common::{
    eyre::{bail, Result},
    glob,
    once_cell::sync::Lazy,
    regex::Regex,
    serde::{Deserialize, Serialize},
    serde_json,
    serde_with::skip_serializing_none,
    strum::Display,
    tokio::{self, sync::Mutex, task::JoinHandle},
    tracing,
};
use events::publish;
use files::{File, FileEvent, Files};
use graph::{Graph, GraphEvent, GraphEventType};
use graph_triples::{resources, Resource};
use path_utils::pathdiff;
use sources::Sources;

use crate::config::CONFIG;
use crate::documents::DOCUMENTS;
use crate::utils::schemas;

#[derive(Debug, Display, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
#[strum(serialize_all = "lowercase", crate = "common::strum")]
enum ProjectEventType {
    Updated,
}

#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
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
    /// Will publish the event under the `projects:<project>:props` topic
    /// so it can be differentiated from `FileEvent`s and `GraphEvent`s for the
    /// same project.
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
#[serde(default, rename_all = "camelCase", crate = "common::serde")]
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
    #[schemars(skip)]
    pub sources: Sources,

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
    pub path: PathBuf,

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

    /// The project's dependency graph
    #[serde(skip_deserializing)]
    #[schemars(schema_with = "Project::schema_graph")]
    pub graph: Graph,
}

impl Project {
    /// Generate the JSON Schema for the `path` property to avoid optionality
    /// due to `skip_deserializing`
    fn schema_path(_generator: &mut SchemaGenerator) -> Schema {
        schemas::typescript("string", true)
    }

    /// Generate the JSON Schema for the `file` property to avoid duplicated
    /// inline type and optionality due to `skip_deserializing`
    fn schema_files(_generator: &mut SchemaGenerator) -> Schema {
        schemas::typescript("Record<string, File>", true)
    }

    /// Generate the JSON Schema for the `graph` property to point to
    /// our custom `Graph` schema
    fn schema_graph(_generator: &mut SchemaGenerator) -> Schema {
        schemas::typescript("Graph", true)
    }

    /// The name of a project's manifest file
    const FILE_NAME: &'static str = "project.json";

    /// The name of a project's storage directory
    const STORAGE_DIR: &'static str = ".stencila";

    /// Load a project's from its manifest file and storage directory
    ///
    /// If there is no manifest file, then default values will be used.
    /// See the `write` method for what gets written to where.
    fn load<P: AsRef<Path>>(path: P) -> Result<Project> {
        let path = path.as_ref();
        if !path.exists() {
            bail!("Project folder does not exist: {}", path.display())
        }
        let path = path.canonicalize()?;

        // Deserialize `.stencila/project.json` file (if any)
        let file = path.join(Project::STORAGE_DIR).join(Project::FILE_NAME);
        let mut project = if file.exists() {
            let json = fs::read_to_string(file)?;
            serde_json::from_str(&json)?
        } else {
            Project::default()
        };

        // Override settings `project.json` file (if any)
        let file = path.join(Project::FILE_NAME);
        if file.exists() {
            let json = fs::read_to_string(file)?;
            let overrides: Project = serde_json::from_str(&json)?;
            project.name = overrides.name;
            project.description = overrides.description;
            project.image = overrides.image;
            project.main = overrides.main;
            project.theme = overrides.theme;
            project.sources = overrides.sources;
            project.watch_exclude_patterns = overrides.watch_exclude_patterns;
        }

        project.path = path;

        Ok(project)
    }

    /// Open a project from an existing folder
    ///
    /// Reads the `project.json` file (if any) and walks the project folder
    /// to build a filesystem tree.
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Project> {
        let path = path.as_ref();

        // Load the project manifest (if any).
        let mut project = Project::load(&path)?;

        // Get all the files and folders in the project
        project.files = Files::new(&path);

        // Update the project's properties, some one which may depend on the files
        // list that we just updated
        project.update(None).await;

        // Attempt to compile the project's graph
        match project.compile().await {
            Ok(..) => (),
            Err(error) => tracing::warn!("While compiling project `{}`: {}", path.display(), error),
        };

        Ok(project)
    }

    /// Initialize a project in a new, or existing, folder
    ///
    /// If the project has already been initialized (i.e. has a manifest file)
    /// then this function will do nothing.
    pub async fn init<P: AsRef<Path>>(path: P) -> Result<()> {
        if path.as_ref().join(Project::FILE_NAME).exists() {
            return Ok(());
        }

        if !path.as_ref().exists() {
            fs::create_dir_all(&path)?;
        }

        let mut project = Project::open(path).await?;
        project.write().await?;

        Ok(())
    }

    /// Read a project's manifest file and update the project
    ///
    /// Overwrites properties with values from the file and then
    /// calls `update()`. If there is no manifest file, then default
    /// values will be used.
    async fn read(&mut self) -> Result<()> {
        let updates = Project::load(&self.path)?;
        self.update(Some(updates)).await;
        Ok(())
    }

    /// Write to a project's manifest file and storage directory
    pub async fn write(&mut self) -> Result<()> {
        self.write_with(None).await
    }

    /// Write to a project's manifest file and storage directory with updates
    ///
    /// If the project folder does not exist yet then it will be created.
    /// Stores the complete project as `./.stencila/project.json` and a trimmed down
    /// version with derived fields removed (e.g. `sources.files`) in `./project.json`.
    pub async fn write_with(&mut self, updates: Option<Project>) -> Result<()> {
        if updates.is_some() {
            self.update(updates).await
        }

        let mut project = serde_json::to_value(&self)?;

        // Write the complete project to the storage directory
        let storage = self.path.join(Project::STORAGE_DIR);
        fs::create_dir_all(&storage)?;
        fs::write(
            storage.join(Project::FILE_NAME),
            serde_json::to_string_pretty(&project)?,
        )?;

        // Trim derived fields
        let project = project.as_object_mut().expect("Expected an object");
        project.remove("path");
        project.remove("imagePath");
        project.remove("mainPath");
        project.remove("files");
        project.remove("graph");
        if let Some(sources) = project.get_mut("sources") {
            let sources = sources.as_array_mut().expect("Expected an array");
            for source in sources.iter_mut() {
                let source = source.as_object_mut().expect("Expected an object");
                source.remove("files");
            }
        }

        // Write the trimmed project to `./project.json`
        let json = serde_json::to_string_pretty(&project)?;
        fs::write(self.path.join(Project::FILE_NAME), json)?;

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
            self.sources = updates.sources;
            self.watch_exclude_patterns = updates.watch_exclude_patterns;
        }

        let files = &self.files.files;

        let config = &CONFIG.lock().await.projects;

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
                None => Some("Untitled".to_string()),
            });

        // Theme defaults to the configured default
        self.theme = self.theme.clone().or_else(|| Some(config.theme.clone()));

        ProjectEvent::publish(self, ProjectEventType::Updated)
    }

    /// Compile a project
    ///
    /// Starts at the main document and walks over related files (linked to, imported from etc)
    /// building up the graph. Also adds sources and their relations to files.
    pub async fn compile(&mut self) -> Result<&mut Project> {
        tracing::debug!("Compiling project: {}", self.path.display());

        let mut graph = Graph::new(self.path.clone());

        // Walk over files starting at the main file
        #[async_recursion::async_recursion]
        async fn walk(visited: &mut Vec<PathBuf>, path: &Path, graph: &mut Graph) -> Result<()> {
            let path_buf = path.to_path_buf();
            if visited.contains(&path_buf) || !path.exists() {
                return Ok(());
            } else {
                visited.push(path_buf);
            }

            let document = DOCUMENTS.open(path, None).await?;
            for (subject, pairs) in document.relations {
                for (relation, object) in pairs {
                    graph.add_triple((subject.clone(), relation, object.clone()));

                    if let Resource::File(file) = object {
                        walk(visited, &file.path, graph).await?;
                    }
                }
            }
            Ok(())
        }
        if let Some(path) = self.main_path.as_ref() {
            graph.add_resource(resources::file(path), None);
            walk(&mut Vec::new(), path, &mut graph).await?;
        }

        // Add sources and relations with associated files
        for source in self.sources.inner.iter() {
            graph.add_resource(source.resource(), None);
            graph.add_triples(source.triples(&self.path))
        }

        // Publish a "graph updated" event
        GraphEvent::publish(&self.path, GraphEventType::Updated, &graph);

        self.graph = graph;
        Ok(self)
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
        let config = &CONFIG.lock().await.projects;
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

            // Should the event trigger a read of the project.json file?
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

            // Should the event trigger a recompilation of the project's graph?
            let should_compile_graph = |_event_path: &Path| {
                // TODO: Filter based on whether the path is in the graph's nodes
                true
            };

            // Compile the project graph
            async fn compile_graph(project: &mut Project) {
                if let Err(error) = project.compile().await {
                    tracing::error!(
                        "While compiling project '{}': {}",
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
                        if should_compile_graph(&path) {
                            compile_graph(project).await;
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
                        if should_compile_graph(&path) {
                            compile_graph(project).await;
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
                        if should_compile_graph(&from) || should_compile_graph(&to) {
                            compile_graph(project).await;
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
                        if should_compile_graph(&path) {
                            compile_graph(project).await;
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
    pub registry: Mutex<HashMap<PathBuf, ProjectHandler>>,
}

impl Projects {
    /// Create a new projects store
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the path of closest project to a path
    ///
    /// Searches up the directory tree for a `project.json` file, returning the parent directory
    /// of the first of those files found. If none is found returns the path (or it's parent directory if
    /// the path is a file).
    pub fn project_of_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
        let path = path.as_ref();
        let current = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        }
        .to_path_buf();
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

    /// Get the path of closest project to the current working directory
    pub fn project_of_cwd() -> Result<PathBuf> {
        Projects::project_of_path(std::env::current_dir()?)
    }

    /// List documents that are currently open
    ///
    /// Returns a vector of document paths (relative to the current working directory)
    pub async fn list(&self) -> Result<Vec<String>> {
        let cwd = std::env::current_dir()?;
        let mut paths = Vec::new();
        for project in self.registry.lock().await.values() {
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
    pub async fn open<P: AsRef<Path>>(&self, path: Option<P>, watch: bool) -> Result<Project> {
        let path = match path {
            Some(path) => path.as_ref().canonicalize()?,
            None => Projects::project_of_cwd()?,
        };

        if let Some(handler) = self.registry.lock().await.get(&path) {
            return Ok(handler.project.lock().await.clone());
        }

        let project = Project::open(&path).await?;
        let handler = ProjectHandler::new(project.clone(), watch).await;
        self.registry.lock().await.insert(path.clone(), handler);
        Ok(project)
    }

    /// Open the current project
    pub async fn current(&self, watch: bool) -> Result<Project> {
        self.open::<PathBuf>(None, watch).await
    }

    /// Get a project
    pub async fn get<P: AsRef<Path>>(&self, path: P) -> Result<Arc<Mutex<Project>>> {
        let path = path.as_ref().canonicalize()?;

        if let Some(handler) = self.registry.lock().await.get(&path) {
            Ok(handler.project.clone())
        } else {
            bail!("No project with path {}", path.display())
        }
    }

    /// Close a project
    pub async fn close<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref().canonicalize()?;
        self.registry.lock().await.remove(&path);
        Ok(())
    }
}

/// The global projects store
pub static PROJECTS: Lazy<Projects> = Lazy::new(Projects::new);

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
    use validator::Validate;

    use common::defaults::Defaults;

    use super::*;

    /// Projects
    ///
    /// Configuration settings for project defaults
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default, rename_all = "camelCase", crate = "common::serde")]
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
#[allow(deprecated)] // Remove when using clap 4.0 (https://github.com/clap-rs/clap/issues/3822)
pub mod commands {
    use cli_utils::{
        clap::{self, Parser},
        result, Result, Run,
    };
    use common::async_trait::async_trait;

    use super::*;

    /// Manage projects
    #[derive(Debug, Parser)]
    pub struct Command {
        #[clap(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, Parser)]
    pub enum Action {
        Init(Init),
        List(List),
        Open(Open),
        Close(Close),
        Show(Show),
        Graph(Graph),
        Schemas(Schemas),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::Init(action) => action.run().await,
                Action::List(action) => action.run().await,
                Action::Open(action) => action.run().await,
                Action::Close(action) => action.run().await,
                Action::Show(action) => action.run().await,
                Action::Graph(action) => action.run().await,
                Action::Schemas(action) => action.run(),
            }
        }
    }

    /// Initialize a project in a new, or existing, folder
    #[derive(Debug, Parser)]
    pub struct Init {
        /// The path of the new, or existing, folder to initialize
        ///
        /// If no folder exists at the path, then one will be created.
        /// If no `project.json` file exists in the folder then a new one
        /// will be created.
        #[clap(default_value = ".")]
        pub folder: PathBuf,
    }

    #[async_trait]
    impl Run for Init {
        async fn run(&self) -> Result {
            Project::init(&self.folder).await?;
            result::nothing()
        }
    }

    /// List open projects
    #[derive(Debug, Parser)]
    pub struct List {}

    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let list = PROJECTS.list().await?;
            result::value(list)
        }
    }

    /// Open a project
    #[derive(Debug, Parser)]
    pub struct Open {
        /// The path of the project folder (defaults to the current project)
        pub folder: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Open {
        async fn run(&self) -> Result {
            let Project { name, .. } = PROJECTS.open(self.folder.clone(), true).await?;
            result::value(name)
        }
    }

    /// Close a project
    #[derive(Debug, Parser)]
    pub struct Close {
        /// The path of the project folder
        #[clap(default_value = ".")]
        pub folder: PathBuf,
    }

    #[async_trait]
    impl Run for Close {
        async fn run(&self) -> Result {
            PROJECTS.close(self.folder.clone()).await?;
            result::nothing()
        }
    }

    /// Show a project details
    #[derive(Debug, Parser)]
    pub struct Show {
        /// The path of the project folder (defaults to the current project)
        pub folder: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.folder.clone(), false).await?;
            let content = project.show()?;
            result::new("md", &content, Some(project))
        }
    }

    /// Output the dependency graph for a project
    ///
    /// Tip: When using the DOT format (the default), if you have GraphViz and ImageMagick
    /// installed you can view the graph by piping the output to them. e.g.
    ///
    /// ```sh
    /// stencila documents graph mydoc.md | dot -Tpng | display
    /// ```
    #[derive(Debug, Parser)]
    #[clap(verbatim_doc_comment)]
    pub struct Graph {
        /// The path of the project folder (defaults to the current project)
        folder: Option<PathBuf>,

        /// The format to output the graph as
        #[clap(long, short, default_value = "dot", possible_values = &graph::FORMATS)]
        to: String,
    }

    #[async_trait]
    impl Run for Graph {
        async fn run(&self) -> Result {
            let project = &mut PROJECTS.open(self.folder.clone(), false).await?;
            let content = project.graph.to_format(&self.to)?;
            result::content(&self.to, &content)
        }
    }

    /// Get JSON schemas for projects and associated types
    #[derive(Debug, Parser)]
    pub struct Schemas {}

    impl Schemas {
        pub fn run(&self) -> Result {
            let schema = schemas()?;
            result::value(schema)
        }
    }
}
