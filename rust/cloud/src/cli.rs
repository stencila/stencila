use cli_utils::{
    clap::{self, Parser},
    common::async_trait::async_trait,
    Result, Run,
};

use crate::{projects, user};

#[derive(Parser)]
pub struct Command {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    Me(user::cli::Me),
    Login(user::cli::Login),
    Logout(user::cli::Logout),
    Projects(projects::cli::Command),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match &self.action {
            Action::Me(action) => action.run().await,
            Action::Login(action) => action.run().await,
            Action::Logout(action) => action.run().await,
            Action::Projects(action) => action.run().await,
        }
    }
}
