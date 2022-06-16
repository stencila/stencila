use cli_utils::{
    clap::{self, Parser},
    common::{async_trait::async_trait, tracing},
    result, Result, Run,
};

use super::*;

/// Manage and use your Stencila account
///
/// Connect
#[derive(Parser)]
pub struct Command {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    Me(Me),
    Login(Login),
    Logout(Logout),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match &self.action {
            Action::Me(action) => action.run().await,
            Action::Login(action) => action.run().await,
            Action::Logout(action) => action.run().await,
        }
    }
}

/// Common options for all actions
#[derive(Parser)]
struct Common {
    /// URL of Stencila
    #[clap(long, short, hide = true)]
    url: Option<String>,
}

/// Show the currently authenticated user
#[derive(Parser)]
#[clap(alias = "user")]
struct Me {
    #[clap(flatten)]
    common: Common,
}

#[async_trait]
impl Run for Me {
    async fn run(&self) -> Result {
        let user = me()?;
        if user.is_none() {
            tracing::info!("No user currently logged in");
        }
        result::value(user)
    }
}

/// Login to your Stencila account
#[derive(Parser)]
#[clap(alias = "signin")]
struct Login {
    #[clap(flatten)]
    common: Common,
}

#[async_trait]
impl Run for Login {
    async fn run(&self) -> Result {
        let user = login(self.common.url.as_deref()).await?;
        result::value(user)
    }
}

/// Logout from your Stencila account
#[derive(Parser)]
#[clap(alias = "signin")]
struct Logout {
    #[clap(flatten)]
    common: Common,
}

#[async_trait]
impl Run for Logout {
    async fn run(&self) -> Result {
        logout().await?;
        result::nothing()
    }
}
