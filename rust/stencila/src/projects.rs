use crate::config::CONFIG;
use crate::conversions::Conversion;
use crate::documents::DOCUMENTS;
use crate::files::{File, Files};
use crate::methods::import::import;
use crate::sources::{self, Source, SourceDestination, SourceTrait};
use events::publish;
use eyre::{bail, Result};
use graph::{Graph, GraphEvent, GraphEventType};
use graph_triples::{resources, Resource};
use notify::DebouncedEvent;
use once_cell::sync::Lazy;
use regex::Regex;
use schemars::gen::SchemaGenerator;
use schemars::schema::{Schema, SchemaObject};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use slug::slugify;
use std::sync::Arc;
use std::time::Duration;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use strum::Display;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use utils::some_string;

#[derive(Debug, Display, JsonSchema, Serialize)]
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
    project: Project,

    /// The type of event
    #[serde(rename = "type")]
    type_: ProjectEventType,
}

impl ProjectEvent {
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
    pub sources: Option<HashMap<String, SourceDestination>>,

    /// A list of file conversions
    conversions: Option<Vec<Conversion>>,

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
    path: PathBuf,

    /// The resolved path of the project's image file
    #[serde(skip_deserializing)]
    image_path: Option<PathBuf>,

    /// The resolved path of the project's main file
    #[serde(skip_deserializing)]
    pub main_path: Option<PathBuf>,

    /// The files in the project folder
    #[serde(skip_deserializing)]
    files: Files,

    /// The project's dependency graph
    #[serde(skip_deserializing)]
    #[schemars(schema_with = "Project::graph_schema")]
    graph: Graph,
}

impl Project {
    /// The name of a project's manifest file, within the project directory
    const FILE_NAME: &'static str = "project.json";

    /// Generate the JSON Schema for the `graph` property
    ///
    /// This is necessary because the JSON Schema for the `Graph` type is handwritten
    /// rather than auo-generated using `schemars` in the `graph` crate.
    fn graph_schema(_generator: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            reference: some_string!("Graph"),
            ..Default::default()
        })
    }

    /// Get the path to a project's manifest file
    fn file<P: AsRef<Path>>(path: P) -> PathBuf {
        path.as_ref().join(Project::FILE_NAME)
    }

    /// The name of a project's storage directory, within the project directory
    const STORAGE_DIR: &'static str = ".stencila";

    /// Get the path to a project's storage directory
    fn storage<P: AsRef<Path>>(path: P) -> PathBuf {
        path.as_ref().join(Project::STORAGE_DIR)
    }

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

        // Deserialize the project from the project.json file (if any)
        let file = Project::file(&path);
        let mut project = if file.exists() {
            let json = fs::read_to_string(file)?;
            serde_json::from_str(&json)?
        } else {
            Project::default()
        };

        // Deserialize details kept in storage, rather than in project.json
        if let Some(sources) = &mut project.sources {
            for (name, source) in sources.iter_mut() {
                let source_file = Project::storage(&path)
                    .join("sources")
                    .join([name, ".json"].concat());
                if source_file.exists() {
                    source.read(source_file)?
                }
            }
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
        let updates = Project::load(&self.path)?;
        self.update(Some(updates)).await;
        Ok(())
    }

    /// Write to a project's manifest file and storage directory.
    ///
    /// If the project folder does not exist yet then it will be created.
    /// Removes derived fields from the project to avoid polluting the project.json file
    /// and instead writes them to the project storage directory.
    pub async fn write(&mut self, updates: Option<Project>) -> Result<()> {
        if updates.is_some() {
            self.update(updates).await
        }

        fs::create_dir_all(&self.path)?;

        let mut project = serde_json::to_value(&self)?;
        let project = project.as_object_mut().expect("Should always be an object");

        if let Some(sources) = project.get_mut("sources") {
            let dir = Project::storage(&self.path).join("sources");
            fs::create_dir_all(&dir)?;

            let sources = sources.as_object_mut().expect("Should always be an object");
            for (name, source) in sources.iter_mut() {
                let source = source.as_object_mut().expect("Should always be an object");
                // Write the whole source to storage
                let file = dir.join([name, ".json"].concat());
                let json = serde_json::to_string_pretty(source)?;
                fs::write(file, json)?;

                // Remove details
                source.remove("files");
            }
        }

        // Remove derived fields
        project.remove("path");
        project.remove("imagePath");
        project.remove("mainPath");
        project.remove("files");
        project.remove("graph");

        let json = serde_json::to_string_pretty(project)?;
        let file = Project::file(&self.path);
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
            graph.add_resource(resources::file(path));
            walk(&mut Vec::new(), path, &mut graph).await?;
        }

        // Add sources and relations with associated files
        if let Some(sources) = self.sources.as_ref() {
            for (name, source) in sources {
                graph.add_resource(resources::source(name));
                graph.add_triples(source.triples(name, &self.path))
            }
        }

        // Add relation for each conversion
        if let Some(conversions) = self.conversions.as_ref() {
            for conversion in conversions {
                graph.add_triple(conversion.triple(&self.path))
            }
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
            sources.insert(name, SourceDestination::new(source.clone(), destination));
        } else {
            let mut sources = HashMap::new();
            sources.insert(name, SourceDestination::new(source.clone(), destination));
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

    /// Get the project graph in some format
    pub fn graph(&self, format: &str) -> Result<String> {
        Ok(match format {
            "dot" => self.graph.to_dot(),
            "json" => serde_json::to_string_pretty(&self.graph)?,
            "yaml" => serde_yaml::to_string(&self.graph)?,
            _ => bail!("Unknown graph format '{}'", format),
        })
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
            None => Projects::current_path()?,
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
pub mod commands {
    use super::*;
    use async_trait::async_trait;
    use cli_utils::{result, Result, Run};
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
        Graph(Graph),
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
    #[async_trait]
    impl Run for Init {
        async fn run(&self) -> Result {
            Project::init(&self.folder).await?;
            result::nothing()
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "List open projects",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}
    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let list = PROJECTS.list().await?;
            result::value(list)
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
    #[async_trait]
    impl Run for Open {
        async fn run(&self) -> Result {
            let Project { name, .. } = PROJECTS.open(self.folder.clone(), true).await?;
            result::value(name)
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
    #[async_trait]
    impl Run for Close {
        async fn run(&self) -> Result {
            PROJECTS.close(self.folder.clone()).await?;
            result::nothing()
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
    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let project = PROJECTS.open(self.folder.clone(), false).await?;
            let content = project.show()?;
            result::new("md", &content, Some(project))
        }
    }

    /// Output a dependency graph for a project
    ///
    /// When using the DOT format, if you have GraphViz and ImageMagick installed
    /// you can view the graph by piping the output to them. For example, to
    /// view a graph of the current project:
    ///
    /// ```sh
    /// stencila projects graph --format dot | dot -Tpng | display
    /// ```
    ///
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Graph {
        /// The path of the project folder (defaults to the current project)
        folder: Option<PathBuf>,

        /// The format to output the graph as
        #[structopt(long, short, default_value = "dot", possible_values = &GRAPH_FORMATS)]
        format: String,
    }

    const GRAPH_FORMATS: [&str; 3] = ["dot", "json", "yaml"];
    #[async_trait]
    impl Run for Graph {
        async fn run(&self) -> Result {
            let project = &mut PROJECTS.open(self.folder.clone(), false).await?;
            let content = project.graph(&self.format)?;
            result::content(&self.format, &content)
        }
    }
}
