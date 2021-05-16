use crate::files::Files;
use eyre::{bail, Result};
use regex::Regex;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    collections::{hash_map::Entry, HashMap},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use strum::{EnumString, EnumVariantNames, ToString, VariantNames};

/// # Details of a project
///
/// An implementation, and extension, of schema.org [`Project`](https://schema.org/Project).
/// Uses schema.org properties where possible but adds extension properties
/// where needed (e.g. `theme`).
#[skip_serializing_none]
#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
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
}

/// A format to display a plugin using
#[derive(Debug, EnumString, EnumVariantNames, PartialEq, ToString)]
#[strum(serialize_all = "lowercase")]
pub enum ShowFormat {
    Md,
    Toml,
    Json,
}

impl Project {
    /// The name of the project's manifest file within the project directory
    const FILE_NAME: &'static str = "project.json";

    /// Get the JSON Schema for a project
    pub fn schema() -> String {
        let schema = schema_for!(Project);
        serde_json::to_string_pretty(&schema).unwrap()
    }

    /// Get the path to a projects' manifest file
    fn file(folder: &str) -> PathBuf {
        Path::new(folder).join(Project::FILE_NAME)
    }

    /// Read a project's manifest file
    ///
    /// If there is no manifest file, then return a default project
    fn read(folder: &str) -> Result<Project> {
        if !Path::new(folder).exists() {
            bail!("Project folder does not exist: {}", folder)
        }

        let file = Project::file(folder);
        let mut project = if file.exists() {
            let json = fs::read_to_string(file)?;
            serde_json::from_str(&json)?
        } else {
            Project::default()
        };
        project.path = Path::new(folder).canonicalize()?;

        Ok(project)
    }

    /// Write a project's manifest file
    ///
    /// If the project folder does not exist yet then it will be created
    pub fn write(folder: &str, project: &Project) -> Result<()> {
        fs::create_dir_all(folder)?;

        let file = Project::file(folder);
        let json = serde_json::to_string_pretty(project)?;
        fs::write(file, json)?;

        Ok(())
    }

    /// Initialize a project in a new, or existing, folder
    ///
    /// If the project has already been initialized (i.e. has a manifest file)
    /// then this function will do nothing.
    pub fn init(folder: &str) -> Result<()> {
        if Project::file(folder).exists() {
            return Ok(());
        }

        let project = Project::default();
        Project::write(folder, &project)?;

        Ok(())
    }

    /// Load a project including creating default values for properties
    /// where necessary
    pub fn open(folder: &str, config: &config::ProjectsConfig, watch: bool) -> Result<Project> {
        let mut project = Project::read(folder)?;

        // Watch exclude patterns default to the configured defaults
        let watch_exclude_patterns = project
            .watch_exclude_patterns
            .clone()
            .unwrap_or_else(|| config.watch_exclude_patterns.clone());

        // Get all the files in the project
        project.files = Files::load(folder, watch, watch_exclude_patterns)?;

        // Resolve the main file path first as some of the other project properties
        // may be defined there (e.g. in the YAML header of a Markdown file)
        project.main_path = project.resolve_main_path(&config.main_patterns);

        // Name defaults to the name of the folder
        project.name = project
            .name
            .or_else(|| match Path::new(folder).components().last() {
                Some(last) => Some(last.as_os_str().to_string_lossy().to_string()),
                None => Some("Unnamed".to_string()),
            });

        // Theme defaults to the configured default
        project.theme = project.theme.or_else(|| Some(config.theme.clone()));

        Ok(project)
    }

    /// Attempt to resolve the path of the main file for a project
    ///
    /// Attempts to use the projects `main` property. If that is not specified, or
    /// there is no matching file in the project, attempts to match one of the
    /// project's files against the `main_patterns`.
    fn resolve_main_path(&self, main_patterns: &[String]) -> Option<PathBuf> {
        let files = &self.files.registry().expect("Unable to get files").files;

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
        for pattern in main_patterns {
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
    }

    /// Show a project in a format
    ///
    /// Used for displaying a project in CLI and possibly elsewhere.
    pub fn show(&self, format: ShowFormat) -> Result<(String, String)> {
        let content = match format {
            ShowFormat::Json => serde_json::to_string_pretty(self)?,
            ShowFormat::Toml => toml::to_string(self)?,
            ShowFormat::Md => {
                use handlebars::Handlebars;

                let template = r#"
# {{name}}

**Main**: {{ main }}
**Theme**: {{ theme }}

"#;
                let hb = Handlebars::new();
                hb.render_template(template.trim(), self)?
            }
        };

        Ok((format.to_string(), content))
    }
}

/// An in-memory store of projects and associated
/// data (e.g. file system watchers)
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Projects {
    /// The projects, stored by absolute path
    pub registry: HashMap<String, Project>,
}

impl Default for Projects {
    fn default() -> Self {
        Projects {
            registry: HashMap::new(),
        }
    }
}

impl Projects {
    /// Get the canonical absolute path of a project folder
    fn path(folder: &str) -> Result<String> {
        Ok(Path::new(folder).canonicalize()?.display().to_string())
    }

    /// List projects that are open
    pub fn list(&self) -> Result<HashMap<String, Project>> {
        Ok(self.registry.clone())
    }

    /// Open a project
    ///
    /// This function `loads` a project, stores it, optionally watches the project folder,
    /// updates the project on changes and publishes the updates on the "project"
    /// pubsub topic channel.
    pub fn open(
        &mut self,
        folder: &str,
        config: &config::ProjectsConfig,
        watch: bool,
    ) -> Result<Project> {
        let path = Projects::path(folder)?;

        let project = match self.registry.entry(path) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => match Project::open(folder, config, watch) {
                Ok(project) => entry.insert(project),
                Err(error) => return Err(error),
            },
        };
        Ok(project.clone())
    }

    /// Close a project
    pub fn close(&mut self, folder: &str) -> Result<()> {
        let path = Projects::path(folder)?;
        self.registry.remove(&path);
        Ok(())
    }
}

#[cfg(feature = "config")]
pub mod config {
    use super::*;
    use defaults::Defaults;
    use validator::Validate;

    /// # Projects
    ///
    /// Configuration settings for project defaults
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default, rename_all = "camelCase")]
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
        Schema(Schema),
    }

    impl Command {
        pub fn run(
            &self,
            projects: &mut Projects,
            config: &config::ProjectsConfig,
        ) -> Result<Option<(String, String)>> {
            let Self { action } = self;
            match action {
                Action::Init(action) => action.run(),
                Action::List(action) => action.run(projects),
                Action::Open(action) => action.run(projects, config),
                Action::Close(action) => action.run(projects),
                Action::Show(action) => action.run(projects, config),
                Action::Schema(action) => action.run(),
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
        pub folder: String,
    }

    impl Init {
        pub fn run(&self) -> Result<Option<(String, String)>> {
            let Self { folder } = self;
            Project::init(folder)?;
            Ok(None)
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "List open projects",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}

    impl List {
        pub fn run(&self, projects: &mut Projects) -> Result<Option<(String, String)>> {
            Ok(Some((
                "json".into(),
                format!("{:?}", projects.list()?.keys()),
            )))
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
        pub folder: String,
    }

    impl Open {
        pub fn run(
            &self,
            projects: &mut Projects,
            config: &config::ProjectsConfig,
        ) -> Result<Option<(String, String)>> {
            let Self { folder } = self;
            projects.open(folder, config, true)?;
            Ok(None)
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
        pub folder: String,
    }

    impl Close {
        pub fn run(&self, projects: &mut Projects) -> Result<Option<(String, String)>> {
            let Self { folder } = self;
            projects.close(folder)?;
            Ok(None)
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
        pub folder: String,

        /// The format to display the project in
        #[structopt(short, long, default_value = "md", possible_values = ShowFormat::VARIANTS, case_insensitive = true)]
        pub format: String,
    }

    impl Show {
        pub fn run(
            &self,
            projects: &mut Projects,
            config: &config::ProjectsConfig,
        ) -> Result<Option<(String, String)>> {
            let Self { folder, format } = self;
            let format = ShowFormat::from_str(&format)?;
            let content = projects.open(folder, config, false)?.show(format)?;
            Ok(Some(content))
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Get the JSON Schema for projects",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Schema {}

    impl Schema {
        pub fn run(&self) -> Result<Option<(String, String)>> {
            Ok(Some(("json".into(), Project::schema())))
        }
    }
}
