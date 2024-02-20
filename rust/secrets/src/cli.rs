use cli_utils::{rpassword::prompt_password, ToStdout};
use common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::Result,
};

use crate::{delete, list, set};

/// Secrets
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// A command to perform with secrets
#[derive(Subcommand)]
enum Command {
    /// List the secrets used by Stencila
    List,

    /// Set a secret used by Stencila
    ///
    /// You will be prompted for the secret
    Set(Set),

    /// Delete a secret previously set using Stencila
    Delete(Delete),
}

#[derive(Args)]
struct Set {
    /// The name of the secret
    name: String,
}

#[derive(Args)]
struct Delete {
    /// The name of the secret
    name: String,
}

impl Cli {
    // Run the CLI
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            list()?.to_stdout();
            return Ok(());
        };

        match command {
            Command::List => list()?.to_stdout(),
            Command::Set(Set { name }) => set(&name, &prompt_password("Enter secret: ")?)?,
            Command::Delete(Delete { name }) => delete(&name)?,
        }

        Ok(())
    }
}
