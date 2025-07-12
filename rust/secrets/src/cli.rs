use std::io::{stdin, IsTerminal, Read};

use ask::ask_for_password;
use cli_utils::{
    color_print::cstr,
    tabulated::{Attribute, Cell, CellAlignment, Color, Tabulated},
    ToStdout,
};
use common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::Result,
};

use crate::{delete, list, name_validator, set};

/// Manage secrets
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all configured secrets</dim>
  <b>stencila secrets</>

  <dim># Set a secret interactively (prompts for value)</dim>
  <b>stencila secrets set</> <g>STENCILA_API_TOKEN</>

  <dim># Set a secret from stdin (pipe the value)</dim>
  <y>echo \"sk-abc123...\"</> <b>|</> <b>stencila secrets set</> <g>OPENAI_API_KEY</>

  <dim># Delete a secret</dim>
  <b>stencila secrets delete</> <g>ANTHROPIC_API_KEY</>

  <dim># Use the add/remove aliases</dim>
  <b>stencila secrets add</> <g>STENCILA_API_TOKEN</>
  <b>stencila secrets remove</> <g>STENCILA_API_TOKEN</>

<bold><b>Security</b></bold>
  Secrets are stored securely using your system's keyring.
  They are used to authenticate with external services like
  AI model providers and cloud platforms.
"
);

pub static SET_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Set a secret interactively (you'll be prompted)</dim>
  <b>stencila secrets set</> <g>OPENAI_API_KEY</>

  <dim># Set a secret from stdin</dim>
  <y>echo \"sk-abc123...\"</> <b>|</> <b>stencila secrets set</> <g>OPENAI_API_KEY</>

  <dim># Set API tokens for different services</dim>
  <b>stencila secrets set</> <g>ANTHROPIC_API_KEY</>
  <b>stencila secrets set</> <g>GOOGLE_AI_API_KEY</>
  <b>stencila secrets set</> <g>STENCILA_API_TOKEN</>

  <dim># Use the add alias</dim>
  <b>stencila secrets add</> <g>STENCILA_API_TOKEN</>

<bold><b>Security</b></bold>
  When setting secrets interactively, your input will be
  hidden. When piping from stdin, ensure your shell history
  doesn't record the command with the secret value.
"
);

pub static DELETE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Delete a specific secret</dim>
  <b>stencila secrets delete</> <g>OPENAI_API_KEY</>

  <dim># Delete API tokens</dim>
  <b>stencila secrets delete</> <g>ANTHROPIC_API_KEY</>
  <b>stencila secrets delete</> <g>GOOGLE_AI_API_KEY</>

  <dim># Use the remove alias</dim>
  <b>stencila secrets remove</> <g>GOOGLE_AI_API_KEY</>

<bold><b>Warning</b></bold>
  This permanently removes the secret from your system's
  keyring. You'll need to set it again if you want to use
  it in the future.
"
);

/// A command to perform with secrets
#[derive(Debug, Subcommand)]
enum Command {
    /// List the secrets used by Stencila
    List,

    /// Set a secret used by Stencila
    ///
    /// You will be prompted for the secret. Alternatively, you can echo the
    /// password into this command i.e. `echo <TOKEN> | stencila secrets set STENCILA_API_TOKEN`
    #[command(alias = "add", after_long_help = SET_AFTER_LONG_HELP)]
    Set(Set),

    /// Delete a secret previously set using Stencila
    #[command(alias = "remove", after_long_help = DELETE_AFTER_LONG_HELP)]
    Delete(Delete),
}

#[derive(Debug, Args)]
struct Set {
    /// The name of the secret
    #[arg(value_parser = name_validator)]
    name: String,
}

#[derive(Debug, Args)]
struct Delete {
    /// The name of the secret
    #[arg(value_parser = name_validator)]
    name: String,
}

impl Cli {
    // Run the CLI
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return list_cli();
        };

        match command {
            Command::List => list_cli()?,
            Command::Set(Set { name }) => {
                let value = if !stdin().is_terminal() {
                    // This allows piping in secrets which can be useful
                    let mut input = String::new();
                    stdin().read_to_string(&mut input)?;
                    input
                } else {
                    ask_for_password(&format!("Enter your {name}")).await?
                };
                set(&name, &value)?
            }
            Command::Delete(Delete { name }) => delete(&name)?,
        }

        Ok(())
    }
}

fn list_cli() -> Result<()> {
    let list = list()?;

    let mut table = Tabulated::new();
    table.set_header(["Name", "Description", "Value"]);
    for secret in list {
        table.add_row([
            Cell::new(&secret.name).add_attribute(Attribute::Bold),
            Cell::new(&secret.description),
            match &secret.redacted {
                Some(redacted) => Cell::new(redacted).fg(Color::Green),
                None => Cell::new(""),
            }
            .set_alignment(CellAlignment::Left),
        ]);
    }

    table.to_stdout();

    Ok(())
}
