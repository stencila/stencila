use crate::cli::display;
use crate::files::{File, FileEvent, Files};
use crate::utils::schemas;
use eyre::{bail, Result};
use regex::Regex;
use schemars::{schema::Schema, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::sync::{mpsc, Arc, Mutex};
use std::{
    collections::{hash_map::Entry, HashMap},
    fs,
    path::{Path, PathBuf},
};

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

    /// Glob patterns for paths to be excluded from file watching
    ///
    /// As a performance optimization, paths that match these patterns are
    /// excluded from file watching updates.
    /// If not specified, will default to the patterns in the
    /// configuration settings.
    watch_exclude_patterns: Option<Vec<String>>,

    // The following properties are derived from the filesystem
    // and should never be read from, or written to, the `project.json` file
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

    /// Generate the JSON Schema for the `file` property to avoid duplicated
    /// inline type and optionality due to `skip_deserializing`
    fn schema_files(_generator: &mut schemars::gen::SchemaGenerator) -> Schema {
        schemas::typescript("Record<string, File>", true)
    }

    /// The name of the project's manifest file within the project directory
    const FILE_NAME: &'static str = "project.json";

    /// Get the path to a project's manifest file
    fn file<P: AsRef<Path>>(folder: P) -> PathBuf {
        folder.as_ref().join(Project::FILE_NAME)
    }

    /// Read a project's manifest file
    ///
    /// If there is no manifest file, then return a default project
    fn read<P: AsRef<Path>>(folder: P) -> Result<Project> {
        let folder = folder.as_ref().canonicalize()?;

        if !folder.exists() {
            bail!("Project folder does not exist: {}", folder.display())
        }

        let file = Project::file(&folder);
        let mut project = if file.exists() {
            let json = fs::read_to_string(file)?;
            serde_json::from_str(&json)?
        } else {
            Project::default()
        };
        project.path = folder;

        Ok(project)
    }

    /// Write a project's manifest file
    ///
    /// If the project folder does not exist yet then it will be created
    pub fn write<P: AsRef<Path>>(folder: P, project: &Project) -> Result<()> {
        fs::create_dir_all(&folder)?;

        let file = Project::file(folder);
        let json = serde_json::to_string_pretty(project)?;
        fs::write(file, json)?;

        Ok(())
    }

    /// Initialize a project in a new, or existing, folder
    ///
    /// If the project has already been initialized (i.e. has a manifest file)
    /// then this function will do nothing.
    pub fn init<P: AsRef<Path>>(folder: P) -> Result<()> {
        if Project::file(&folder).exists() {
            return Ok(());
        }

        let project = Project::default();
        Project::write(folder, &project)?;

        Ok(())
    }

    /// Open a project from an existing folder
    ///
    /// Reads the `project.json` file (if any) and walks the project folder
    /// to build a filesystem tree.
    pub fn open<P: AsRef<Path>>(folder: P, config: &config::ProjectsConfig) -> Result<Project> {
        let mut project = Project::read(&folder)?;

        // Get all the files and folders in the project
        project.files = Files::new(folder);

        // Update the project's properties, some one which may depend on the files
        project.update(config);

        Ok(project)
    }

    /// Update a project after changes to it's package.json or main file.
    ///
    /// Attempts to use the projects `main` property. If that is not specified, or
    /// there is no matching file in the project, attempts to match one of the
    /// project's files against the `main_patterns`.
    fn update(&mut self, config: &config::ProjectsConfig) {
        let files = &self.files.files;

        // Resolve the main file path first as some of the other project properties
        // may be defined there (e.g. in the YAML header of a Markdown file)
        self.main_path = (|| {
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
**Main file**: {{ main_path }}
**Image file**: {{ image_path }}
**Theme**: {{ theme }}

"#;
        let hb = Handlebars::new();
        let md = hb.render_template(template.trim(), self)?;
        Ok(md)
    }
}

#[derive(Debug)]
pub struct ProjectHandler {
    /// The project being handled
    project: Arc<Mutex<Project>>,

    /// The project watcher's channel sender
    ///
    /// Held so that when this handler is dropped, the
    /// watcher thread is closed
    pub watcher: Option<crossbeam_channel::Sender<()>>,
}

impl ProjectHandler {
    /// Open a project and, optionally, watch it for changes
    pub fn open<P: AsRef<Path>>(
        folder: P,
        config: &config::ProjectsConfig,
        watch: bool,
    ) -> Result<ProjectHandler> {
        let project = Project::open(&folder, config)?;

        // Watch exclude patterns default to the configured defaults.
        // Note that this project setting can not be updated while it is open.
        let watch_exclude_patterns = project
            .watch_exclude_patterns
            .clone()
            .unwrap_or_else(|| config.watch_exclude_patterns.clone());

        let project = Arc::new(Mutex::new(project));

        let watcher = if watch {
            // Clone the mutex to send to the thread
            let project = project.clone();

            // Create a `PathBuf` of the folder to send to the thread
            let folder = folder.as_ref().to_path_buf();

            // Create a `string` of the folder for use by logging in the thread
            let folder_string = folder.display().to_string();

            let (thread_sender, thread_receiver) = crossbeam_channel::bounded(1);
            std::thread::spawn(move || -> Result<()> {
                use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
                use std::time::Duration;

                let (watcher_sender, watcher_receiver) = mpsc::channel();
                let mut watcher = watcher(watcher_sender, Duration::from_secs(1))?;
                watcher.watch(&folder, RecursiveMode::Recursive).unwrap();

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

                let should_include = |event_path: &Path| {
                    if let Ok(event_path) = event_path.strip_prefix(&folder) {
                        for glob in &exclude_globs {
                            if glob.matches(&event_path.display().to_string()) {
                                return false;
                            }
                        }
                    }
                    true
                };

                let handle_event = |event: DebouncedEvent| match event {
                    DebouncedEvent::Create(path) => {
                        if should_include(&path) {
                            let project = &mut *project.lock().unwrap();
                            project.files.created(&path)
                        }
                    }
                    DebouncedEvent::Remove(path) => {
                        if should_include(&path) {
                            let project = &mut *project.lock().unwrap();
                            project.files.removed(&path)
                        }
                    }
                    DebouncedEvent::Rename(from, to) => {
                        if should_include(&from) || should_include(&to) {
                            let project = &mut *project.lock().unwrap();
                            project.files.renamed(&from, &to);
                        }
                    }
                    DebouncedEvent::Write(path) => {
                        if should_include(&path) {
                            let project = &mut *project.lock().unwrap();
                            project.files.modified(&path)
                        }
                    }
                    _ => {}
                };

                let span = tracing::info_span!("file_watch", project = folder_string.as_str());
                let _enter = span.enter();
                tracing::debug!("Starting project file watch: {}", folder_string);
                // Event checking timeout. Can be quite long since only want to check
                // whether we can end this thread.
                let timeout = Duration::from_millis(100);
                loop {
                    // Check for an event. Use `recv_timeout` so we don't
                    // get stuck here and will do following check that ends this
                    // thread if the owning `ProjectHandler` is dropped
                    if let Ok(event) = watcher_receiver.recv_timeout(timeout) {
                        handle_event(event)
                    }
                    // Check to see if this thread should be ended
                    if let Err(crossbeam_channel::TryRecvError::Disconnected) =
                        thread_receiver.try_recv()
                    {
                        tracing::debug!("Ending project file watch: {}", folder_string);
                        break;
                    }
                }

                Ok(())
            });

            Some(thread_sender)
        } else {
            None
        };

        Ok(ProjectHandler { project, watcher })
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
    pub fn new() -> Self {
        Self::default()
    }

    /// List documents that are currently open
    ///
    /// Returns a vector of document paths (relative to the current working directory)
    pub fn list(&self) -> Result<Vec<String>> {
        let cwd = std::env::current_dir()?;
        let mut paths = Vec::new();
        for project in self.registry.values() {
            let path = &project.project.lock().expect("Unable to obtain lock").path;
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
    pub fn open<P: AsRef<Path>>(
        &mut self,
        folder: P,
        config: &config::ProjectsConfig,
        watch: bool,
    ) -> Result<Project> {
        let path = folder.as_ref().canonicalize()?;

        let handler = match self.registry.entry(path) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => match ProjectHandler::open(folder, config, watch) {
                Ok(handler) => entry.insert(handler),
                Err(error) => return Err(error),
            },
        };

        Ok(handler.project.lock().unwrap().clone())
    }

    /// Close a project
    pub fn close<P: AsRef<Path>>(&mut self, folder: P) -> Result<()> {
        let path = folder.as_ref().canonicalize()?;
        self.registry.remove(&path);
        Ok(())
    }
}

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![
        schemas::generate::<Project>()?,
        schemas::generate::<File>()?,
        schemas::generate::<FileEvent>()?,
    ]);
    Ok(schemas)
}

#[cfg(feature = "config")]
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
        pub fn run(
            &self,
            projects: &mut Projects,
            config: &config::ProjectsConfig,
        ) -> display::Result {
            let Self { action } = self;
            match action {
                Action::Init(action) => action.run(),
                Action::List(action) => action.run(projects),
                Action::Open(action) => action.run(projects, config),
                Action::Close(action) => action.run(projects),
                Action::Show(action) => action.run(projects, config),
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
        pub fn run(&self) -> display::Result {
            let Self { folder } = self;
            Project::init(folder)?;
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
        pub fn run(&self, projects: &mut Projects) -> display::Result {
            let list = projects.list()?;
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
        /// The path of the project folder
        #[structopt(default_value = ".")]
        pub folder: PathBuf,
    }

    impl Open {
        pub fn run(
            &self,
            projects: &mut Projects,
            config: &config::ProjectsConfig,
        ) -> display::Result {
            let Self { folder } = self;
            let Project { name, .. } = projects.open(folder, config, true)?;
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
        /// The path of the project folder
        #[structopt(default_value = ".")]
        pub folder: PathBuf,
    }

    impl Show {
        pub fn run(
            &self,
            projects: &mut Projects,
            config: &config::ProjectsConfig,
        ) -> display::Result {
            let Self { folder } = self;
            let project = projects.open(folder, config, false)?;
            let content = project.show()?;
            display::new("md", &content, Some(project))
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
