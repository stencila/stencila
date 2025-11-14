use clap::{Args, Parser, Subcommand};
use eyre::{Result, bail};
use textwrap::{Options, termwidth, wrap};
use url::Url;

use stencila_ask::{Answer, AskLevel, AskOptions, ask_for_password, ask_with};
use stencila_cli_utils::{
    color_print::{cformat, cstr},
    message,
};
use stencila_cloud::TokenSource;
use stencila_server::{ServeOptions, get_server_token};

/// Manage Stencila Cloud account
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Check your cloud authentication status</dim>
  <b>stencila cloud status</>

  <dim># Sign in to Stencila Cloud</dim>
  <b>stencila cloud signin</>

  <dim># Sign out from Stencila Cloud</dim>
  <b>stencila cloud signout</>

  <dim># View logs from a cloud workspace session</dim>
  <b>stencila cloud logs --session</> <g>SESSION_ID</>

  <dim># Create a site for the current workspace</dim>
  <b>stencila cloud site create</>

  <dim># Delete the workspace site</dim>
  <b>stencila cloud site delete</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Status(Status),
    Signin(Signin),
    Signout(Signout),
    Logs(Logs),
    Site(Site),
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
            Command::Logs(logs) => logs.run().await,
            Command::Site(site) => site.run().await,
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
    async fn run(self) -> Result<()> {
        let status = stencila_cloud::status();

        match (status.token, status.token_source) {
            (Some(redacted_token), Some(source)) => {
                message(
                    &format!(
                        "Signed in to Stencila Cloud\n Access token: {redacted_token} (set via {source})\n"
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
pub struct Signin {
    /// Signin by manually entering a Stencila access token
    #[arg(long, short)]
    manual: bool,
}

pub static SIGNIN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Sign in to Stencila Cloud</dim>
  <b>stencila cloud signin</>

  <dim># Sign in manually</dim>
  <b>stencila cloud signin --manual</>

  <dim># Use one of the command aliases</dim>
  <b>stencila signin</>
  <b>stencila login</>
"
);

impl Signin {
    pub async fn run(self) -> Result<()> {
        if self.manual {
            let token = ask_for_password(cstr!(
                "Enter an access token from <b>https://stencila.cloud/access-tokens</>"
            ))
            .await?;
            stencila_cloud::signin(&token)?;

            return Ok(());
        }

        // Get (or generate) an access token so it can be included in the URL
        let server_token = get_server_token();

        // Serve with access token
        let options = ServeOptions {
            server_token: Some(server_token.clone()),
            no_startup_message: true,
            ..Default::default()
        };

        let serve = tokio::spawn(async move { stencila_server::serve(options).await });

        // Open the browser to the Stencila Cloud CLI signin page with a callback
        // to the ~auth endpoint.
        let mut callback = Url::parse("http://127.0.0.1:9000/~auth/callback")?;
        callback.query_pairs_mut().append_pair("sst", &server_token);
        let url = format!("https://stencila.cloud/signin/cli?callback={callback}");

        message(
            cstr!("Opening browser to signin at <b>https://stencila.cloud</>"),
            Some("☁️"),
        );
        webbrowser::open(&url)?;

        // Await the serve task (it will stop gracefully when auth_success triggers shutdown)
        match serve.await {
            Ok(Ok(())) => {
                message("✅ Signed in successfully!", None);
            }
            Ok(Err(error)) => bail!(error),
            Err(error) => bail!(error),
        }

        Ok(())
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
    pub async fn run(self) -> Result<()> {
        let status_before = stencila_cloud::signout()?;

        match (status_before.token, status_before.token_source) {
            (Some(_), Some(TokenSource::Keyring)) => message(
                "Signed out from Stencila Cloud
 Access token removed from keyring",
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

/// Display logs from Stencila Cloud workspace sessions
#[derive(Debug, Args)]
#[command(after_long_help = LOGS_AFTER_LONG_HELP)]
pub struct Logs {
    /// The session ID to retrieve logs for
    #[arg(long, short)]
    session: String,

    /// Maximum number of recent logs to display
    #[arg(long, short)]
    limit: Option<usize>,

    /// Continuously poll for new logs every N seconds (press Ctrl+C to stop)
    ///
    /// If provided without a value, defaults to 5 seconds. Minimum value is 1 second.
    #[arg(
        long,
        short,
        default_missing_value = "5",
        num_args = 0..=1,
        value_parser = clap::value_parser!(u64).range(1..)
    )]
    follow: Option<u64>,

    /// Filter logs by level (error, warn, info, debug, trace)
    #[arg(long)]
    level: Option<String>,
}

pub static LOGS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># View logs for a session</dim>
  <b>stencila cloud logs --session SESSION_ID</>

  <dim># View last 50 logs</dim>
  <b>stencila cloud logs --session SESSION_ID --limit 50</>

  <dim># Follow logs (poll every 5 seconds by default)</dim>
  <b>stencila cloud logs --session SESSION_ID --follow</>

  <dim># Follow logs with custom polling interval</dim>
  <b>stencila cloud logs --session SESSION_ID --follow 10</>

  <dim># Filter logs by level</dim>
  <b>stencila cloud logs --session SESSION_ID --level error</>
"
);

impl Logs {
    /// Format a timestamp to have exactly 2 decimal places for subseconds
    fn format_timestamp(timestamp: &str) -> String {
        // Find the decimal point in the timestamp
        // Expected format: 2025-11-05T07:21:47.546473193Z
        if let (Some(dot_pos), Some(z_pos)) = (timestamp.rfind('.'), timestamp.rfind('Z')) {
            // Extract the parts
            let before_dot = &timestamp[..dot_pos];
            let subseconds = &timestamp[dot_pos + 1..z_pos];

            // Take only first 2 digits of subseconds
            let truncated = if subseconds.len() >= 2 {
                &subseconds[..2]
            } else {
                subseconds
            };

            return format!("{}.{}Z", before_dot, truncated);
        }

        // Fall back to original if format doesn't match expected pattern
        timestamp.to_string()
    }

    /// Remove redundant timestamp from message if it starts with the same timestamp
    fn trim_message_timestamp(timestamp: &str, message: &str) -> String {
        // Extract timestamp up to seconds (YYYY-MM-DDTHH:MM:SS)
        // Expected format: 2025-11-05T07:21:47.546473193Z
        if let Some(dot_pos) = timestamp.find('.') {
            let timestamp_to_seconds = &timestamp[..dot_pos];

            // Check if message starts with this timestamp pattern
            if let Some(rest) = message.strip_prefix(timestamp_to_seconds) {
                return rest.to_string();
            }
        }

        // Return original message if no match
        message.to_string()
    }

    #[allow(clippy::print_stdout)]
    pub async fn run(self) -> Result<()> {
        use tokio::time::{Duration, sleep};

        let mut last_log_count = 0;

        loop {
            // Fetch logs from API
            let logs = stencila_cloud::get_logs(&self.session).await?;

            // Filter by level if specified
            let logs: Vec<_> = if let Some(ref level) = self.level {
                logs.into_iter()
                    .filter(|log| log.level.eq_ignore_ascii_case(level))
                    .collect()
            } else {
                logs
            };

            // In follow mode, only show new logs after the first fetch
            let logs_to_display = if self.follow.is_some() {
                if logs.len() > last_log_count {
                    &logs[last_log_count..]
                } else {
                    &[]
                }
            } else {
                &logs[..]
            };

            // Apply limit if specified (only on first non-follow display)
            let logs_to_display = if self.follow.is_none() && self.limit.is_some() {
                let limit = self.limit.unwrap_or(logs_to_display.len());
                if logs_to_display.len() > limit {
                    &logs_to_display[logs_to_display.len() - limit..]
                } else {
                    logs_to_display
                }
            } else {
                logs_to_display
            };

            // Display logs
            for log in logs_to_display {
                // Format timestamp and calculate visual width
                let timestamp = Self::format_timestamp(&log.timestamp);

                // Create styled prefix and matching indent for wrapped lines
                let initial_indent = cformat!("<dim>{timestamp}</dim> ");
                let subsequent_indent = " ".repeat(timestamp.len() + 1);

                // Trim redundant timestamp from message if present, then trim whitespace
                let message = Self::trim_message_timestamp(&log.timestamp, &log.message);
                let message = message.trim();

                // Wrap lines
                let width = termwidth();
                let options = Options::new(width)
                    .initial_indent(&initial_indent)
                    .subsequent_indent(&subsequent_indent);
                let wrapped_lines = wrap(message, options);

                // Print first line with prefix, subsequent lines with indent
                for line in wrapped_lines.iter() {
                    println!("{line}");
                }
            }

            // Update last log count for follow mode
            last_log_count = logs.len();

            // If not following, exit after first display
            let Some(poll_interval) = self.follow else {
                break;
            };

            // Wait before polling again
            sleep(Duration::from_secs(poll_interval)).await;
        }

        Ok(())
    }
}

/// Manage Stencila Sites
#[derive(Debug, Parser)]
#[command(alias = "sites", after_long_help = SITE_AFTER_LONG_HELP)]
pub struct Site {
    #[command(subcommand)]
    command: SiteCommand,
}

pub static SITE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create a site for the workspace</dim>
  <b>stencila cloud site create</>

  <dim># Delete the workspace site</dim>
  <b>stencila cloud site delete</>
"
);

#[derive(Debug, Subcommand)]
enum SiteCommand {
    Create(SiteCreate),
    Delete(SiteDelete),
}

impl Site {
    pub async fn run(self) -> Result<()> {
        match self.command {
            SiteCommand::Create(create) => create.run().await,
            SiteCommand::Delete(delete) => delete.run().await,
        }
    }
}

/// Create a new Stencila Site
#[derive(Debug, Args)]
#[command(after_long_help = SITE_CREATE_AFTER_LONG_HELP)]
pub struct SiteCreate {
    /// Path to the workspace directory where .stencila/site.yaml will be created
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static SITE_CREATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create site for the current workspace</dim>
  <b>stencila cloud site create</>

  <dim># Create site for another workspace</dim>
  <b>stencila cloud site create --path /path/to/project</>
"
);

impl SiteCreate {
    pub async fn run(self) -> Result<()> {
        let path = self
            .path
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        let (config, already_existed) = stencila_cloud::sites::ensure_site(&path).await?;
        let url = config.default_url();

        if already_existed {
            message(
                &format!("Site already exists for workspace: {url}"),
                Some("ℹ️"),
            );
        } else {
            message(
                &format!("Site successfully created for workspace: {url}"),
                Some("✅"),
            );
        }

        Ok(())
    }
}

/// Delete a Stencila Site
#[derive(Debug, Args)]
#[command(after_long_help = SITE_DELETE_AFTER_LONG_HELP)]
pub struct SiteDelete {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static SITE_DELETE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Delete site for current workspace</dim>
  <b>stencila cloud site delete</>

  <dim># Delete site for another workspace</dim>
  <b>stencila cloud site delete --path /path/to/project</>
"
);

impl SiteDelete {
    pub async fn run(self) -> Result<()> {
        let path = self
            .path
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        // Ask for confirmation with warning level
        let answer = ask_with(
            "This will permanently delete the site on Stencila Cloud including all content. This cannot be undone.",
            AskOptions {
                level: AskLevel::Warning,
                default: Some(Answer::No),
                title: Some("Delete Stencila Site".into()),
                yes_text: Some("Yes, delete".into()),
                no_text: Some("Cancel".into()),
                ..Default::default()
            },
        )
        .await?;

        if !answer.is_yes() {
            message("Site deletion cancelled", Some("ℹ️"));
            return Ok(());
        }

        stencila_cloud::sites::delete_site(&path).await?;

        message("Site deleted successfully", Some("✅"));

        Ok(())
    }
}
