//! The main file for the `stencila` CLI tool
//!
//! This module requires the `cli` feature to be enabled e.g.
//!
//! cargo run --no-default-features --features cli help

#![recursion_limit = "256"]

use async_trait::async_trait;
use cli_utils::{result, Result, Run};
use std::{collections::HashMap, path::PathBuf};
use stencila::{
    config::{self, CONFIG},
    documents::{self, DOCUMENTS},
    logging::{
        self,
        config::{LoggingConfig, LoggingStdErrConfig},
        LoggingFormat, LoggingLevel,
    },
    projects::{self, PROJECTS},
    sources,
};
use structopt::StructOpt;
use strum::VariantNames;

/// Stencila, in a terminal console, on your own machine
///
/// Enter interactive mode by using the `--interact` option with any command.
#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder,
    setting = structopt::clap::AppSettings::ColoredHelp,
    setting = structopt::clap::AppSettings::VersionlessSubcommands
)]
pub struct Args {
    /// The command to run
    #[structopt(subcommand)]
    pub command: Option<Command>,

    /// Format to display results of commands (e.g. json, yaml, md)
    ///
    /// If the command result can be displayed in the specified format
    /// it will be. Display format preferences can be configured.
    #[structopt(long, global = true, alias = "as")]
    pub display: Option<String>,

    /// Enter interactive mode (with any command and options as the prefix)
    #[structopt(short, long, global = true, alias = "interactive")]
    pub interact: bool,

    /// Print debug level log events and additional diagnostics
    ///
    /// Equivalent to setting `--log-level=debug` and `--log-format=detail`.
    /// Overrides the both of those options and any configuration settings
    /// for logging on standard error stream.
    #[structopt(long, global = true)]
    pub debug: bool,

    /// The minimum log level to print
    #[structopt(long, global = true, possible_values = LoggingLevel::VARIANTS, case_insensitive = true)]
    pub log_level: Option<LoggingLevel>,

    /// The format to print log events
    #[structopt(long, global = true, possible_values = LoggingFormat::VARIANTS, case_insensitive = true)]
    pub log_format: Option<LoggingFormat>,
}

/// Global arguments that should be removed when entering interactive mode
/// because they can only be set / are relevant at startup. Other global arguments,
/// which need to be accessible at the line level, should be added to `interact::Line` below.
pub const GLOBAL_ARGS: [&str; 6] = [
    "-i",
    "--interact",
    "--interactive",
    "--debug",
    "--log-level",
    "--log-format",
];

#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder
)]
pub enum Command {
    // General commands that delegate to either the `projects` module,
    // or the `documents` module (depending upon if path is a folder or file),
    // or combine results from both in the case of `List`.
    List(ListCommand),
    Open(OpenCommand),
    Close(CloseCommand),
    Show(ShowCommand),
    Convert(ConvertCommand),
    Diff(DiffCommand),
    Merge(MergeCommand),

    // The special `with` command which enters interactive mode with
    // `projects <placeholder> <path>` or `documents <placeholder> <path>`
    // as the command prefix
    With(WithCommand),

    // Module-specific commands defined in the `stencila` library
    #[structopt(aliases = &["project"])]
    Projects(projects::commands::Command),

    #[structopt(aliases = &["document", "docs", "doc"])]
    Documents(documents::commands::Command),

    #[structopt(aliases = &["source"])]
    Sources(sources::commands::Command),

    #[structopt(aliases = &["codec"])]
    Codecs(codecs::commands::Command),

    #[structopt(aliases = &["parser"])]
    Parsers(parsers::commands::Command),

    #[structopt(aliases = &["kernel"])]
    Kernels(kernels::commands::Command),

    Config(config::commands::Command),

    #[cfg(feature = "binaries")]
    #[structopt(aliases = &["binary"])]
    Binaries(binaries::commands::Command),

    #[cfg(feature = "plugins")]
    #[structopt(aliases = &["plugin"])]
    Plugins(stencila::plugins::commands::Command),

    #[cfg(feature = "upgrade")]
    Upgrade(stencila::upgrade::commands::Command),

    #[cfg(feature = "serve")]
    #[structopt(aliases = &["serve"])]
    Server(stencila::serve::commands::Command),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match self {
            Command::List(command) => command.run().await,
            Command::Open(command) => command.run().await,
            Command::Close(command) => command.run().await,
            Command::Show(command) => command.run().await,
            Command::Convert(command) => command.run().await,
            Command::Diff(command) => command.run().await,
            Command::Merge(command) => command.run().await,
            Command::With(command) => command.run().await,
            Command::Documents(command) => command.run().await,
            Command::Projects(command) => command.run().await,
            Command::Sources(command) => command.run().await,
            Command::Codecs(command) => command.run().await,
            Command::Parsers(command) => command.run().await,
            Command::Kernels(command) => command.run().await,
            Command::Config(command) => command.run().await,

            #[cfg(feature = "binaries")]
            Command::Binaries(command) => command.run().await,
            #[cfg(feature = "plugins")]
            Command::Plugins(command) => command.run().await,
            #[cfg(feature = "upgrade")]
            Command::Upgrade(command) => command.run().await,
            #[cfg(feature = "serve")]
            Command::Server(command) => command.run().await,
        }
    }
}

// The structopt used in interactive mode
#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::NoBinaryName,
    setting = structopt::clap::AppSettings::ColoredHelp,
    setting = structopt::clap::AppSettings::VersionlessSubcommands
)]
pub struct Line {
    #[structopt(subcommand)]
    pub command: Command,

    /// Display format
    ///
    /// The format used to display results of commands (if possible)
    #[structopt(long, global = true, alias = "as")]
    pub display: Option<String>,
}

#[async_trait]
impl Run for Line {
    /// Run the command
    async fn run(&self) -> Result {
        self.command.run().await
    }

    /// Run the command and print it to the console
    ///
    /// This override allow the user to specify preferred display format
    /// on a per line basis.
    async fn print(&self, formats: &[String]) {
        match self.run().await {
            Ok(value) => {
                let mut formats: Vec<String> = formats.into();
                if let Some(display) = &self.display {
                    formats.insert(0, display.clone())
                }
                if let Err(error) = result::print::value(value, &formats) {
                    result::print::error(error)
                }
            }
            Err(error) => result::print::error(error),
        }
    }
}

/// List all open project and documents.
#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::NoBinaryName,
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
pub struct ListCommand {}
#[async_trait]
impl Run for ListCommand {
    async fn run(&self) -> Result {
        let mut value = HashMap::new();
        value.insert("projects", PROJECTS.list().await?);
        value.insert("documents", DOCUMENTS.list().await?);
        result::value(value)
    }
}

/// Open a project or document using a web browser
///
/// If the path is a file, it will be opened as a document.
/// If the path is a folder, it will be opened as a project and it's main file
/// (if any) opened.
///
/// In the future this command will open the project/document
/// in the Stencila Desktop if that is available.
#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::NoBinaryName,
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
pub struct OpenCommand {
    /// The file or folder to open
    path: Option<PathBuf>,

    /// The theme to use to view the document
    #[structopt(short, long)]
    theme: Option<String>,
}
#[async_trait]
impl Run for OpenCommand {
    #[cfg(feature = "serve")]
    async fn run(&self) -> Result {
        let (is_project, path) = match &self.path {
            Some(path) => (path.is_dir(), path.clone()),
            None => (true, std::env::current_dir()?),
        };

        let path = if is_project {
            let project = PROJECTS.open(Some(path), true).await?;
            match project.main_path {
                Some(path) => path,
                None => {
                    tracing::info!("Project has no main document to display");
                    return result::nothing();
                }
            }
        } else {
            let document = DOCUMENTS.open(&path, None).await?;
            document.path
        };

        if cfg!(feature = "webbrowser") {
            // Given that the URL (and thus token) may be visible by other processes from the arguments passed
            // to the "open the browser command" use a short-expiry, single-use token.
            let url = stencila::serve::serve(&path, Some(15), true).await?;
            tracing::info!("Opening in browser {}", url);
            webbrowser::open(&url)?;
        } else {
            // Provide the user with a URL containing a longer-expiry, single-use token.
            let url = stencila::serve::serve(&path, Some(3600), true).await?;
            tracing::info!("Available at {}", url)
        }

        // If not in interactive mode then just sleep here forever to avoid finishing
        if std::env::var("STENCILA_INTERACT_MODE").is_err() {
            use tokio::time::{sleep, Duration};
            sleep(Duration::MAX).await;
        }

        result::nothing()
    }

    #[cfg(not(feature = "serve"))]
    async fn run(self, _context: &mut Context) -> Result {
        bail!("The `serve` feature has not been enabled")
    }
}

/// Close a project or document
///
/// If the path is a file, the associated open document (if any) will be closed.
/// If the path is a folder, the associated project (if any) will be closed.
/// Closing a document or project just means that it is unloaded from memory
/// and the file or folder is not longer watched for changes.
#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::NoBinaryName,
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
pub struct CloseCommand {
    /// The file or folder to close
    #[structopt(default_value = ".")]
    path: PathBuf,
}
#[async_trait]
impl Run for CloseCommand {
    async fn run(&self) -> Result {
        if self.path.is_dir() {
            PROJECTS.close(&self.path).await?;
        } else {
            DOCUMENTS.close(&self.path).await?;
        }

        result::nothing()
    }
}

/// Show a project or document
///
/// If the path is a file, it will be opened as a document and displayed.
/// If the path is a folder, it will be opened as a project and displayed.
#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::NoBinaryName,
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
pub struct ShowCommand {
    /// The file or folder to close
    path: Option<PathBuf>,
}
#[async_trait]
impl Run for ShowCommand {
    async fn run(&self) -> Result {
        if let Some(path) = &self.path {
            if path.is_file() {
                return result::value(DOCUMENTS.open(&path, None).await?);
            }
        }

        result::value(PROJECTS.open(self.path.clone(), true).await?)
    }
}

/// Currently, these commands simply delegate to the `documents` module
type ConvertCommand = documents::commands::Convert;
type DiffCommand = documents::commands::Diff;
type MergeCommand = documents::commands::Merge;

/// Run commands interactively with a particular project or document
///
#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::NoBinaryName,
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
pub struct WithCommand {
    /// The file or folder to run command with
    path: PathBuf,
}
#[async_trait]
impl Run for WithCommand {
    async fn run(&self) -> Result {
        result::nothing()
    }
}

/// Main entry point function
#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    #[cfg(all(windows, feature = "cli-pretty"))]
    #[allow(unused_must_use)]
    {
        cli_utils::ansi_term::enable_ansi_support();
    }

    let args: Vec<String> = std::env::args().collect();

    // Parse args into a command
    let parsed_args = Args::from_iter_safe(args.clone());
    let Args {
        command,
        display,
        debug,
        log_level,
        log_format,
        interact,
        ..
    } = match parsed_args {
        Ok(args) => args,
        Err(error) => {
            // An argument parsing error happened, possibly because the user
            // provided incomplete command prefix to interactive mode. Handle that.
            if args.contains(&"-i".to_string())
                || args.contains(&"--interact".to_string())
                || args.contains(&"--interactive".to_string())
            {
                Args {
                    command: None,
                    display: None,
                    debug: args.contains(&"--debug".to_string()),
                    log_level: None,
                    log_format: None,
                    interact: true,
                }
            } else {
                // Print the error `clap` help or usage message and exit
                eprintln!("{}", error);
                std::process::exit(exitcode::USAGE);
            }
        }
    };

    // Create a preliminary logging subscriber to be able to log any issues
    // when reading the logging config.
    let prelim_subscriber_guard = logging::prelim();
    let logging_config = CONFIG.lock().await.logging.clone();
    drop(prelim_subscriber_guard);

    // Create a logging config with local overrides
    let logging_config = LoggingConfig {
        stderr: LoggingStdErrConfig {
            level: if debug {
                LoggingLevel::Debug
            } else {
                log_level.unwrap_or(logging_config.stderr.level)
            },
            format: if debug {
                LoggingFormat::Detail
            } else {
                log_format.unwrap_or(logging_config.stderr.format)
            },
        },
        ..logging_config
    };

    // To ensure all log events get written to file, take guards here, so that
    // non blocking writers do not get dropped until the end of this function.
    // See https://tracing.rs/tracing_appender/non_blocking/struct.workerguard
    let _logging_guards = logging::init(true, false, true, &logging_config)?;

    // Set up error reporting and progress indicators for better feedback to user
    #[cfg(feature = "clis-pretty")]
    {
        // Setup `color_eyre` crate for better error reporting with span and back traces
        if std::env::var("RUST_SPANTRACE").is_err() {
            std::env::set_var("RUST_SPANTRACE", if debug { "1" } else { "0" });
        }
        if std::env::var("RUST_BACKTRACE").is_err() {
            std::env::set_var("RUST_BACKTRACE", if debug { "1" } else { "0" });
        }
        cli_utils::color_eyre::config::HookBuilder::default()
            .display_env_section(false)
            .install()?;

        // Subscribe to progress events and display them on console
        use stencila::pubsub::{subscribe, Subscriber};
        subscribe(
            "progress",
            Subscriber::Function(cli_utils::progress::subscriber),
        )?;
    }

    // If not explicitly upgrading then run an upgrade check in the background
    #[cfg(feature = "upgrade")]
    let upgrade_thread = {
        if let Some(Command::Upgrade(_)) = command {
            None
        } else {
            Some(stencila::upgrade::upgrade_auto())
        }
    };

    // Use the desired display format, falling back to configured values
    let formats = match display {
        Some(display) => vec![display],
        None => vec!["md".to_string(), "yaml".to_string(), "json".to_string()],
    };

    // The `with` command is always interactive; need to work out
    // if projects or documents module
    #[allow(unused_variables)]
    let (interact, module) = match &command {
        Some(Command::With(WithCommand { path })) => (
            true,
            if path.is_dir() {
                "projects".to_string()
            } else {
                "documents".to_string()
            },
        ),
        _ => (interact, "".to_string()),
    };

    // Run the command and print result
    if let (false, Some(command)) = (interact, command) {
        command.print(&formats).await;
    } else {
        #[cfg(feature = "cli-interact")]
        {
            let mut prefix: Vec<String> = args
                .into_iter()
                // Remove executable name
                .skip(1)
                // Remove the global args which can not be applied to each interactive line
                .filter(|arg| !GLOBAL_ARGS.contains(&arg.as_str()))
                .collect();

            // Insert the module if this is the `with` command
            if !module.is_empty() {
                prefix.insert(0, module);
            }

            let history = config::dir(true)?.join("history.txt");
            std::env::set_var("STENCILA_INTERACT_MODE", "1");
            cli_utils::interact::run::<Line>(prefix, &formats, &history).await?;
        }
        #[cfg(not(feature = "cli-interact"))]
        {
            eprintln!("Compiled with `interact` feature disabled.");
            std::process::exit(exitcode::USAGE);
        }
    };

    // Join the upgrade thread and log any errors
    #[cfg(feature = "upgrade")]
    if let Some(upgrade_thread) = upgrade_thread {
        if let Err(_error) = upgrade_thread.await.await {
            tracing::warn!("Error while attempting to join upgrade thread")
        }
    }

    Ok(())
}
