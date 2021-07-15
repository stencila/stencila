use defaults::Defaults;
use eyre::{bail, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{EnumIter, ToString};

use crate::utils::schemas;

#[derive(PartialEq, Clone, Debug, ToString, EnumIter, JsonSchema, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Source {
    GitHub(GitHub),
}

pub fn resolve(source: &str) -> Result<Source> {
    Ok(Source::GitHub(GitHub::default()))
}

#[skip_serializing_none]
#[derive(PartialEq, Clone, Debug, JsonSchema, Deserialize, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct SourceDestination {
    /// The name of this source / destination.
    ///
    /// A unique identifier within the project, mainly for convenient
    /// removal or re-import from the command line.
    pub name: String,

    /// The source
    pub source: Source,

    /// The destination path within the project
    pub destination: Option<String>,
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

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![schemas::generate::<Source>()?]);
    Ok(schemas)
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use crate::{cli::display, projects::Projects};
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

    impl Command {
        pub async fn run(self, projects: &mut Projects) -> display::Result {
            let Self { action } = self;
            match action {
                Action::List(action) => action.run(projects).await,
                Action::Add(action) => action.run(projects).await,
                Action::Remove(action) => action.run(projects).await,
                Action::Import(action) => action.run(projects).await,
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

    impl List {
        pub async fn run(&self, projects: &mut Projects) -> display::Result {
            let project = projects.current(false).await?;
            display::value(project.sources)
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

    impl Add {
        pub async fn run(self, projects: &mut Projects) -> display::Result {
            let Self {
                source,
                destination,
                name,
            } = self;
            let mut project = projects.current(false).await?;
            let files = project.add_source(&source, destination, name).await?;
            display::value(files)
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

    impl Remove {
        pub async fn run(self, projects: &mut Projects) -> display::Result {
            let Self { name } = self;
            let mut project = projects.current(false).await?;
            project.remove_source(&name).await?;
            display::nothing()
        }
    }

    /// Import a source into the current project
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Import {
        /// A URL or other identifier of the source
        pub source: String,

        /// The path to import the source to
        pub path: Option<String>,
    }

    impl Import {
        pub async fn run(self, projects: &mut Projects) -> display::Result {
            let Self { source, path } = self;
            let mut project = projects.current(false).await?;
            let files = project.import_source(&source, path).await?;
            display::value(files)
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Get JSON Schemas for sources",
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
