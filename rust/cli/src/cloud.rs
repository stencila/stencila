use cli_utils::{color_print::cstr, message};
use common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::Result,
};

use cloud::TokenSource;

/// Manage Stencila Cloud account
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  // TODO: complete as for other module's CLI_AFTER_LONG_HELP
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Status(Status),
    Signin(Signin),
    Signout(Signout),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return Status.run().await;
        };

        match command {
            Command::Status(status) => status.run().await,
            Command::Signin(signin) => signin.run().await,
            Command::Signout(signout) => signout.run().await,
        }
    }
}

/// Display Stencila Cloud authentication status
#[derive(Debug, Args)]
#[command(after_long_help = STATUS_AFTER_LONG_HELP)]
struct Status;

pub static STATUS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># See your current authentication status</dim>
  <b>stencila cloud status</>
"
);

impl Status {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        let status = cloud::status();

        match (status.token, status.token_source) {
            (Some(redacted_token), Some(source)) => {
                message(
                    &format!(
                        "Signed in to Stencila Cloud\nToken: {redacted_token} (set via {source})\n"
                    ),
                    Some("✅"),
                );
                message(cstr!("To sign out, run <b>stencila signout</>"), Some("💡"));
            }
            (None, None) => {
                message("Not signed in to Stencila Cloud\n", Some("❌"));
                message(
                    cstr!("To sign in, run <b>stencila cloud signin</>"),
                    Some("💡"),
                );
            }
            _ => {
                message!("⚠️  Unknown authentication status");
            }
        }

        Ok(())
    }
}

/// Sign in to Stencila Cloud
#[derive(Debug, Args)]
#[command(alias = "login", after_long_help = SIGNIN_AFTER_LONG_HELP)]
pub struct Signin;

pub static SIGNIN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Sign in to Stencila Cloud</dim>
  <b>stencila cloud signin</>

  <dim># Use one of the command aliases</dim>
  <b>stencila signin</>
  <b>stencila login</>
"
);

impl Signin {
    #[allow(clippy::print_stderr)]
    pub async fn run(self) -> Result<()> {
        todo!()
    }
}

/// Sign out from Stencila Cloud
#[derive(Debug, Args)]
#[command(alias = "logout", after_long_help = SIGNOUT_AFTER_LONG_HELP)]
pub struct Signout;

pub static SIGNOUT_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Sign out from Stencila Cloud</dim>
  <b>stencila cloud signout</>

  <dim># Use one of the command aliases</dim>
  <b>stencila signout</>
  <b>stencila logout</>
"
);

impl Signout {
    #[allow(clippy::print_stderr)]
    pub async fn run(self) -> Result<()> {
        let status_before = cloud::signout()?;

        match (status_before.token, status_before.token_source) {
            (Some(_), Some(TokenSource::Keyring)) => message(
                "Signed out from Stencila Cloud
Token removed from keyring",
                Some("✅"),
            ),
            (Some(_), Some(TokenSource::EnvVar)) => {
                message(
                    "Cannot sign out: token is set via environment variable.\n",
                    Some("⚠️"),
                );
                message(
                cstr!(
                    "To sign out, remove the <b>STENCILA_API_TOKEN</> environment variable from your shell profile or system environment."
                ),
                Some("💡"),
            )
            }
            (None, None) => {
                message!("ℹ️  Already signed out from Stencila Cloud");
            }
            _ => {
                message!("⚠️  Unknown authentication status during sign out");
            }
        }

        Ok(())
    }
}
