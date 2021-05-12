use eyre::{bail, Result};
use regex::Regex;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{collections::HashMap, str::FromStr};
use std::{
    fs,
    path::{Path, PathBuf},
};
use strum::{EnumString, EnumVariantNames, ToString, VariantNames};

/// # A file or directory within a project
#[skip_serializing_none]
#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct File {
    /// The relative path of the file within the project folder
    path: String,

    /// Whether the entry is a directory or not
    is_dir: bool,

    /// The media type (aka MIME type) of the file
    media_type: Option<String>,

    /// The SHA1 hash of the contents of the file
    sha1: Option<String>,
}

impl File {
    pub fn from_path(folder: &Path, path: &Path) -> (PathBuf, File) {
        let canonical_path = path.canonicalize().expect("Unable to canonicalize path");
        let relative_path = path
            .strip_prefix(folder)
            .expect("Unable to strip prefix")
            .display()
            .to_string();

        let media_type = mime_guess::from_path(path)
            .first()
            .map(|mime| mime.essence_str().to_string());

        let file = File {
            path: relative_path,
            is_dir: path.is_dir(),
            media_type,
            ..Default::default()
        };

        (canonical_path, file)
    }
}

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

    /// URL of the image to be used when displaying the project
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

    /// The files in the project directory
    files: HashMap<PathBuf, File>,
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
        let project = if file.exists() {
            let json = fs::read_to_string(file)?;
            serde_json::from_str(&json)?
        } else {
            Project::default()
        };

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
    /// then this function will do nothing
    pub fn init(folder: &str) -> Result<Project> {
        if Project::file(folder).exists() {
            return Project::read(folder);
        }

        let project = Project::default();
        Project::write(folder, &project)?;

        Ok(project)
    }

    /// Load a project including creating default values for properties
    /// where necessary
    pub fn load(folder: &str, config: &config::ProjectsConfig) -> Result<Project> {
        let project = Project::read(folder)?;
        let Project {
            name, main, theme, ..
        } = project;

        // Name defaults to the name of the folder
        let name = name.or_else(|| match Path::new(folder).components().last() {
            Some(last) => Some(last.as_os_str().to_string_lossy().to_string()),
            None => Some("Unnamed".to_string()),
        });

        // Get all the files in the project
        let folder = Path::new(folder);
        let files: Vec<(PathBuf, File)> = walkdir::WalkDir::new(folder)
            .into_iter()
            .filter_map(|entry| {
                let entry = match entry.ok() {
                    Some(entry) => entry,
                    None => return None,
                };
                Some(File::from_path(folder, entry.path()))
            })
            .collect();

        // Main defaults to the first file that matches configured patterns (if any)
        let main = main.or_else(|| {
            // See if there are any files matching patterns
            for pattern in &config.main_patterns {
                let re = match Regex::new(&pattern.to_lowercase()) {
                    Ok(re) => re,
                    Err(_) => {
                        tracing::warn!("Project main file pattern is invalid: {}", pattern);
                        continue;
                    }
                };
                for (_, file) in &files {
                    let File { is_dir, path, .. } = file;
                    if *is_dir {
                        continue;
                    }
                    if re.is_match(path) {
                        return Some(path.clone());
                    }
                }
            }

            None
        });

        let files = files.into_iter().collect();

        // Name defaults to the configured default
        let theme = theme.or_else(|| Some(config.theme.clone()));

        Ok(Project {
            name,
            main,
            theme,
            files,
            ..project
        })
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
    pub projects: HashMap<String, Project>,
}

impl Default for Projects {
    fn default() -> Self {
        Projects {
            projects: HashMap::new(),
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
        Ok(self.projects.clone())
    }

    /// Open a project
    ///
    /// This function `loads` a project, stores it, watches the project folder,
    /// updates the project on changes and publishes the updates on the "projects"
    /// pubsub topic channel.
    pub fn open(
        &mut self,
        folder: &str,
        config: &config::ProjectsConfig,
    ) -> Result<(String, Project)> {
        let path = Projects::path(folder)?;
        let project = Project::load(folder, config)?;
        self.projects.insert(path.clone(), project.clone());
        Ok((path, project))
    }

    /// Close a project
    pub fn close(&mut self, folder: &str) -> Result<()> {
        let path = Projects::path(folder)?;
        self.projects.remove(&path);
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
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

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
        pub fn run(&self) -> Result<()> {
            let Self { folder } = self;
            Project::init(folder)?;
            Ok(())
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
        pub fn run(&self, config: &config::ProjectsConfig) -> Result<(String, String)> {
            let Self { folder, format } = self;
            let format = ShowFormat::from_str(&format)?;
            Project::load(folder, config)?.show(format)
        }
    }
}
