use cli_utils::{
    clap::{self, Parser},
    common::async_trait::async_trait,
    result, Result, Run,
};

use crate::taskfile::Taskfile;

/// Manage and run project tasks
#[derive(Parser)]
#[clap(alias = "task")]
pub struct Command {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Parser)]
pub enum Action {
    Init(Init),
    List(List),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match &self.action {
            Action::Init(action) => action.run().await,
            Action::List(action) => action.run().await,
        }
    }
}

/// Initialize tasks
///
/// Creates a new `Taskfile.yaml` in a directory.
#[derive(Parser)]
pub struct Init {}

#[async_trait]
impl Run for Init {
    async fn run(&self) -> Result {
        let taskfile = Taskfile {
            ..Default::default()
        };
        taskfile.write_current()?;

        result::nothing()
    }
}

/// List tasks
///
/// Use this command to get a list of tasks in the current project.
#[derive(Parser)]
pub struct List {}

#[async_trait]
impl Run for List {
    async fn run(&self) -> Result {
        result::nothing()
    }
}
