//! CLI commands for testing and inspecting the state of the application
//!
//! These commands provide interfaces to lower level functions
//! and state of the application. They are useful for developers wanting
//! to debug, or gain more insight into the workings of, the application.
//! However, they could be distracting if placed in other "main" command
//! listings and so are pulled into and grouped under a generic `inspect`
//! command.

#[cfg(feature = "cli")]
pub mod cli {
    use eyre::Result;
    use structopt::StructOpt;

    use crate::{plugins, request};

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Inspect the state of the application",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub struct Args {
        #[structopt(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub enum Action {
        Methods(plugins::cli::Methods),
        Delegate(plugins::cli::Delegate),
        Request(request::cli::Args),
    }

    pub async fn run(args: Args, plugins: &mut plugins::Plugins) -> Result<()> {
        let Args { action } = args;

        match action {
            Action::Methods(methods) => methods.run(plugins).await,
            Action::Delegate(delegate) => delegate.run(plugins).await,
            Action::Request(args) => request::cli::run(args).await,
        }
    }
}
