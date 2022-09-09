use cli_utils::{
    clap::{self, Parser},
    common::async_trait::async_trait,
    result, Result, Run,
};

use crate::docs;

/// Manage and run project tasks
#[derive(Parser)]
#[clap(alias = "task")]
pub struct Command {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Parser)]
pub enum Action {
    Docs(Docs_),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match &self.action {
            Action::Docs(action) => action.run().await,
        }
    }
}

/// Generate docs for configuration options
#[derive(Parser)]
#[clap(hide = true)]
pub struct Docs_;

#[async_trait]
impl Run for Docs_ {
    async fn run(&self) -> Result {
        docs::generate()?;
        result::nothing()
    }
}
