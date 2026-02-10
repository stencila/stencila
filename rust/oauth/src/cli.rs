//! CLI subcommands for OAuth login flows.

use clap::{Args, Parser, Subcommand, ValueEnum};
use eyre::Result;
use stencila_cli_utils::{color_print::cstr, message};

use crate::persist;

/// Manage OAuth authentication for AI model providers
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Check which providers you are logged in to</dim>
  <b>stencila oauth</>

  <dim># Login to Anthropic via OAuth</dim>
  <b>stencila oauth login</> <g>anthropic</>

  <dim># Login to GitHub Copilot</dim>
  <b>stencila oauth login</> <g>copilot</>

  <dim># Logout from a provider</dim>
  <b>stencila oauth logout</> <g>gemini</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Status(Status),
    Login(Login),
    Logout(Logout),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return Status.run().await;
        };

        match command {
            Command::Status(status) => status.run().await,
            Command::Login(login) => login.run().await,
            Command::Logout(logout) => logout.run().await,
        }
    }
}

// ---------------------------------------------------------------------------
// Provider enum
// ---------------------------------------------------------------------------

/// An OAuth-capable AI model provider.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Provider {
    Anthropic,
    Copilot,
    Gemini,
    Openai,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic => write!(f, "anthropic"),
            Self::Copilot => write!(f, "copilot"),
            Self::Gemini => write!(f, "gemini"),
            Self::Openai => write!(f, "openai"),
        }
    }
}

impl Provider {
    fn secret_key(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::Copilot => "copilot",
            Self::Gemini => "gemini",
            Self::Openai => "openai",
        }
    }
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

/// Display OAuth authentication status for all providers
#[derive(Debug, Args)]
#[command(after_long_help = STATUS_AFTER_LONG_HELP)]
struct Status;

pub static STATUS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># See which providers you are logged in to</dim>
  <b>stencila oauth status</>
  <b>stencila oauth</>
"
);

impl Status {
    async fn run(self) -> Result<()> {
        let providers = [
            Provider::Anthropic,
            Provider::Copilot,
            Provider::Gemini,
            Provider::Openai,
        ];

        let mut any_logged_in = false;
        for provider in providers {
            let has_creds = persist::load_credentials(provider.secret_key())?.is_some();
            if has_creds {
                message!("  {} {}", "\u{2705}", provider);
                any_logged_in = true;
            } else {
                message!("  {} {}", "\u{274c}", provider);
            }
        }

        if !any_logged_in {
            message("\n\u{1f4a1} To login, run *stencila oauth login PROVIDER*");
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Login
// ---------------------------------------------------------------------------

/// Login to an AI model provider via OAuth
#[derive(Debug, Args)]
#[command(alias = "signin", after_long_help = LOGIN_AFTER_LONG_HELP)]
struct Login {
    /// The provider to login to
    provider: Provider,
}

pub static LOGIN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Login to Anthropic</dim>
  <b>stencila oauth login</> <g>anthropic</>

  <dim># Login to GitHub Copilot</dim>
  <b>stencila oauth login</> <g>copilot</>

  <dim># Login to Google Gemini</dim>
  <b>stencila oauth login</> <g>gemini</>

  <dim># Login to OpenAI</dim>
  <b>stencila oauth login</> <g>openai</>
"
);

impl Login {
    async fn run(self) -> Result<()> {
        match self.provider {
            #[cfg(feature = "anthropic")]
            Provider::Anthropic => {
                message!("\u{1f310} Starting Anthropic OAuth login...");
                crate::anthropic::login().await?;
                message!("\u{2705} Logged in to Anthropic");
            }
            #[cfg(feature = "copilot")]
            Provider::Copilot => {
                message!("\u{1f310} Starting GitHub Copilot login...");
                crate::copilot::login().await?;
                message!("\u{2705} Logged in to GitHub Copilot");
            }
            #[cfg(feature = "gemini")]
            Provider::Gemini => {
                message!("\u{1f310} Starting Google Gemini OAuth login...");
                crate::gemini::login().await?;
                message!("\u{2705} Logged in to Google Gemini");
            }
            #[cfg(feature = "openai")]
            Provider::Openai => {
                message!("\u{1f310} Starting OpenAI OAuth login...");
                crate::openai::login().await?;
                message!("\u{2705} Logged in to OpenAI");
            }
            #[allow(unreachable_patterns)]
            _ => {
                message!(
                    "\u{26a0}\u{fe0f} The {} provider is not enabled in this build",
                    self.provider
                );
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Logout
// ---------------------------------------------------------------------------

/// Logout from an AI model provider (remove stored OAuth credentials)
#[derive(Debug, Args)]
#[command(alias = "signout", after_long_help = LOGOUT_AFTER_LONG_HELP)]
struct Logout {
    /// The provider to logout from
    provider: Provider,
}

pub static LOGOUT_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Logout from Anthropic</dim>
  <b>stencila oauth logout</> <g>anthropic</>

  <dim># Logout from GitHub Copilot</dim>
  <b>stencila oauth logout</> <g>copilot</>
"
);

impl Logout {
    async fn run(self) -> Result<()> {
        let key = self.provider.secret_key();

        if persist::load_credentials(key)?.is_some() {
            persist::delete_credentials(key)?;
            message!("\u{2705} Logged out from {}", self.provider);
        } else {
            message!("\u{2139}\u{fe0f} Not logged in to {}", self.provider);
        }

        Ok(())
    }
}
