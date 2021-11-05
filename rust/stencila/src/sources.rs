use crate::{
    graphs::{relations, resources, Triple},
    utils::schemas,
};
use async_trait::async_trait;
use defaults::Defaults;
use enum_dispatch::enum_dispatch;
use eyre::{bail, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{fs, path::Path};
use strum::{Display, EnumIter, IntoEnumIterator};

/// Trait for project sources. This allows us to use `enum_dispatch` to
/// dispatch these methods based on the type of source.
#[async_trait]
#[enum_dispatch]
pub trait SourceTrait {
    /// Attempt to resole the source from an identifier
    fn resolve(&self, id: &str) -> Option<Source>;

    /// Generate a default name for the source
    ///
    /// Generated names do not need to be unique (that is guaranteed elsewhere)
    /// and should simply provide an recognizable, relatively short way to refer
    /// to a source
    fn default_name(&self) -> String;
}

/// A project source
#[enum_dispatch(SourceTrait)]
#[derive(
    PartialEq, Clone, Debug, Display, Defaults, EnumIter, JsonSchema, Deserialize, Serialize,
)]
#[def = "Null(Null{})"]
#[serde(tag = "type")]
pub enum Source {
    /// A null variant that exists only so that we can define a default source
    Null(Null),
    Elife(Elife),
    GitHub(GitHub),
}

/// Resolve a source from an identifier
///
/// Returns the first source that matches the identifier
pub fn resolve(id: &str) -> Result<Source> {
    for source in Source::iter() {
        if let Some(source) = source.resolve(id) {
            return Ok(source);
        }
    }
    bail!(
        "Unable to resolve the identifier '{}' into a project source",
        id
    )
}

/// A source-destination combination
///
/// Each source by destination combination should be unique to a project.
/// It is possible to have the same source being imported to multiple
/// destinations within a project and for multiple sources to used the same
/// destination (e.g. the root directory of the project).
#[skip_serializing_none]
#[derive(PartialEq, Clone, Debug, Defaults, JsonSchema, Deserialize, Serialize)]
#[serde(default)]
#[schemars(deny_unknown_fields)]
pub struct SourceDestination {
    /// The source from which files will be imported
    pub source: Source,

    /// The destination path within the project
    pub destination: Option<String>,

    /// Whether or not the source is active
    ///
    /// If the source is active an import job will be created for it
    /// each time the project is updated.
    #[def = "true"]
    active: bool,

    /// A list of file paths currently associated with the source,
    /// relative to the project root
    pub files: Option<Vec<String>>,
}

impl SourceDestination {
    /// Create a new `SourceDestination`
    pub fn new(source: Source, destination: Option<String>) -> SourceDestination {
        SourceDestination {
            source,
            destination,
            ..Default::default()
        }
    }

    /// Read a `SourceDestination` from a JSON file
    ///
    /// Only changes the properties that are NOT saved in the project.json file.
    pub fn read<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            bail!("Project source file does not exist: {}", path.display())
        }
        let path = path.canonicalize()?;

        let json = fs::read_to_string(path)?;
        let source: SourceDestination = serde_json::from_str(&json)?;

        self.files = source.files;
        Ok(())
    }

    /// Generate a set of graph triples describing relation between the source
    /// and it's associated files.
    pub fn triples(&self, name: &str, project: &Path) -> Vec<Triple> {
        match &self.files {
            Some(files) => files
                .iter()
                .map(|file| {
                    (
                        resources::source(name),
                        relations::imports(self.active),
                        resources::file(&project.join(file)),
                    )
                })
                .collect(),
            None => Vec::new(),
        }
    }
}

#[derive(PartialEq, Clone, Debug, Defaults, JsonSchema, Deserialize, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Null {}

impl SourceTrait for Null {
    fn resolve(&self, _id: &str) -> Option<Source> {
        None
    }

    fn default_name(&self) -> String {
        "null".to_string()
    }
}

#[skip_serializing_none]
#[derive(PartialEq, Clone, Debug, Defaults, JsonSchema, Deserialize, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Elife {
    /// Number of the article
    pub article: u32,
}

impl SourceTrait for Elife {
    fn resolve(&self, id: &str) -> Option<Source> {
        static SIMPLE_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^elife:(?://)?(\d+)").expect("Unable to create regex"));

        static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(?:https?://)?elifesciences\.org/articles/(\d+).*$")
                .expect("Unable to create regex")
        });

        let article = if let Some(captures) = SIMPLE_REGEX.captures(id) {
            Some(captures[1].parse().unwrap_or_default())
        } else {
            URL_REGEX
                .captures(id)
                .map(|captures| captures[1].parse().unwrap_or_default())
        };

        article.map(|article| Source::Elife(Elife { article }))
    }

    fn default_name(&self) -> String {
        format!("elife-{}", self.article)
    }
}

#[skip_serializing_none]
#[derive(PartialEq, Clone, Debug, Defaults, JsonSchema, Deserialize, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct GitHub {
    /// Owner of the repository
    pub owner: String,

    /// Name of the repository
    pub name: String,

    /// Path within the repository
    pub path: Option<String>,
}

impl SourceTrait for GitHub {
    fn resolve(&self, id: &str) -> Option<Source> {
        static SIMPLE_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^github:(?://)?([a-z0-9\-]+)/([a-z0-9\-_]+)(?:/(.+))?$")
                .expect("Unable to create regex")
        });

        static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^(?:https?://)?github\.com/([a-z0-9\-]+)/([a-z0-9\-_]+)/?(?:(?:tree|blob)/(?:[^/]+)/(.+))?$")
                .expect("Unable to create regex")
        });

        if let Some(captures) = SIMPLE_REGEX.captures(id) {
            Some(Source::GitHub(GitHub {
                owner: captures[1].to_string(),
                name: captures[2].to_string(),
                path: captures.get(3).map(|group| group.as_str().to_string()),
            }))
        } else {
            URL_REGEX.captures(id).map(|captures| {
                Source::GitHub(GitHub {
                    owner: captures[1].to_string(),
                    name: captures[2].to_string(),
                    path: captures.get(3).map(|group| group.as_str().to_string()),
                })
            })
        }
    }

    fn default_name(&self) -> String {
        format!("github-{}-{}", self.owner, self.name)
    }
}

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![schemas::generate::<SourceDestination>()?]);
    Ok(schemas)
}

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use crate::projects::PROJECTS;
    use async_trait::async_trait;
    use cli_utils::{result, Result, Run};
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage the current project's sources",
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
        List(List),
        Add(Add),
        Remove(Remove),
        Import(Import),
        Schemas(Schemas),
    }
    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::List(action) => action.run().await,
                Action::Add(action) => action.run().await,
                Action::Remove(action) => action.run().await,
                Action::Import(action) => action.run().await,
                Action::Schemas(action) => action.run(),
            }
        }
    }

    /// List the sources for the current project
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}
    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let project = PROJECTS.current(false).await?;
            result::value(project.sources)
        }
    }

    /// Add a source to the current project
    ///
    /// Does not import the source use the `import` command for that.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Add {
        /// A URL or other identifier of the source
        pub source: String,

        /// The path to import the source to
        pub destination: Option<String>,

        /// The name to give the source
        pub name: Option<String>,
    }
    #[async_trait]
    impl Run for Add {
        async fn run(&self) -> Result {
            let mut project = PROJECTS.current(false).await?;
            let files = project
                .add_source(&self.source, self.destination.clone(), self.name.clone())
                .await?;
            result::value(files)
        }
    }

    /// Remove a source from the current project
    ///
    /// Note that this will remove a files imported from this source
    /// into the project.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Remove {
        /// Name of the source to remove
        pub name: String,
    }
    #[async_trait]
    impl Run for Remove {
        async fn run(&self) -> Result {
            let mut project = PROJECTS.current(false).await?;
            project.remove_source(&self.name).await?;
            result::nothing()
        }
    }

    /// Import a source into the current project
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Import {
        /// A name or other identifier of the source
        pub source: String,

        /// The path to import the source to
        pub destination: Option<String>,
    }
    #[async_trait]
    impl Run for Import {
        async fn run(&self) -> Result {
            let mut project = PROJECTS.current(false).await?;
            let files = project
                .import_source(&self.source, self.destination.clone())
                .await?;
            result::value(files)
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Get JSON Schemas for sources",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Schemas {}

    impl Schemas {
        pub fn run(&self) -> Result {
            let schema = schemas()?;
            result::value(schema)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn elife_resolve() -> Result<()> {
        assert_eq!(
            resolve("elife:52258")?,
            Source::Elife(Elife { article: 52258 })
        );

        assert_eq!(
            resolve("elife://52258")?,
            Source::Elife(Elife { article: 52258 })
        );

        assert_eq!(
            resolve("https://elifesciences.org/articles/52258")?,
            Source::Elife(Elife { article: 52258 })
        );

        assert_eq!(
            resolve("elifesciences.org/articles/52258")?,
            Source::Elife(Elife { article: 52258 })
        );

        Ok(())
    }

    #[test]
    fn github_resolve() -> Result<()> {
        assert_eq!(
            resolve("github:owner/name")?,
            Source::GitHub(GitHub {
                owner: "owner".to_string(),
                name: "name".to_string(),
                path: None
            })
        );

        assert_eq!(
            resolve("github:owner/name/some/path/in/repo.md")?,
            Source::GitHub(GitHub {
                owner: "owner".to_string(),
                name: "name".to_string(),
                path: Some("some/path/in/repo.md".to_string())
            })
        );

        assert_eq!(
            resolve("https://github.com/owner/name/")?,
            Source::GitHub(GitHub {
                owner: "owner".to_string(),
                name: "name".to_string(),
                path: None
            })
        );

        assert_eq!(
            resolve("https://github.com/owner/name/tree/master/some/path/in/repo.md")?,
            Source::GitHub(GitHub {
                owner: "owner".to_string(),
                name: "name".to_string(),
                path: Some("some/path/in/repo.md".to_string())
            })
        );

        assert_eq!(
            resolve("https://github.com/owner/name/blob/master/some/path/in/repo.md")?,
            Source::GitHub(GitHub {
                owner: "owner".to_string(),
                name: "name".to_string(),
                path: Some("some/path/in/repo.md".to_string())
            })
        );

        Ok(())
    }
}
