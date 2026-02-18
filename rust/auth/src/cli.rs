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
  <b>stencila auth</>

  <dim># Login to Anthropic via OAuth</dim>
  <b>stencila auth login</> <g>anthropic</>

  <dim># Login to GitHub Copilot</dim>
  <b>stencila auth login</> <g>copilot</>

  <dim># Logout from a provider</dim>
  <b>stencila auth logout</> <g>gemini</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Status(Status),
    Login(Login),
    Logout(Logout),
}

impl Cli {
    /// Run the auth CLI command.
    ///
    /// # Errors
    ///
    /// Returns an error if the subcommand fails.
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return Status.run();
        };

        match command {
            Command::Status(status) => status.run(),
            Command::Login(login) => login.run().await,
            Command::Logout(logout) => logout.run(),
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
  <b>stencila auth status</>
  <b>stencila auth</>
"
);

impl Status {
    #[allow(clippy::unused_self)]
    fn run(self) -> Result<()> {
        let providers = [
            Provider::Anthropic,
            Provider::Copilot,
            Provider::Gemini,
            Provider::Openai,
        ];

        let mut any_authenticated = false;
        for provider in providers {
            let has_oauth = persist::load_credentials(provider.secret_key())?.is_some();
            let detected_source = detect_external_credentials(provider);

            if has_oauth {
                message!("  {} {}", "\u{2705}", provider);
                any_authenticated = true;
            } else if let Some(source) = detected_source {
                message!("  {} {} <dim>(via {})</>", "\u{2705}", provider, source);
                any_authenticated = true;
            } else {
                message!("  {} {}", "\u{274c}", provider);
            }
        }

        if !any_authenticated {
            message("\n\u{1f4a1} To login, run *stencila auth login PROVIDER*");
        }

        Ok(())
    }
}

/// Check whether external tools (Claude Code, Codex CLI) provide
/// credentials for a given provider. Returns a human-readable source
/// name when detected, `None` otherwise.
fn detect_external_credentials(provider: Provider) -> Option<&'static str> {
    match provider {
        Provider::Anthropic => crate::claude_code::load_credentials().map(|_| "Claude Code"),
        Provider::Openai => crate::codex_cli::load_credentials().map(|_| "Codex CLI"),
        _ => None,
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
  <b>stencila auth login</> <g>anthropic</>

  <dim># Login to GitHub Copilot</dim>
  <b>stencila auth login</> <g>copilot</>

  <dim># Login to Google Gemini</dim>
  <b>stencila auth login</> <g>gemini</>

  <dim># Login to OpenAI</dim>
  <b>stencila auth login</> <g>openai</>
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
  <b>stencila auth logout</> <g>anthropic</>

  <dim># Logout from GitHub Copilot</dim>
  <b>stencila auth logout</> <g>copilot</>
"
);

impl Logout {
    fn run(self) -> Result<()> {
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
