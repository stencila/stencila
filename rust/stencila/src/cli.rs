//! The main file for the `stencila` CLI tool
//!
//! This module requires the `cli` feature to be enabled e.g.
//!
//! cargo run --no-default-features --features cli help

use std::{collections::HashMap, path::PathBuf};

use cli_utils::{
    clap::{self, AppSettings, Parser},
    result, stderr_isatty, Result, Run,
};
use common::{
    async_trait::async_trait,
    eyre::{self, bail},
    strum::VariantNames,
    tokio, tracing,
};
use tasks::Taskfile;
use utils::some_string;

use crate::{
    config::{self, CONFIG},
    documents::{self, DOCUMENTS},
    logging::{
        self,
        config::{LoggingConfig, LoggingStdErrConfig},
        LoggingFormat, LoggingLevel,
    },
    projects::PROJECTS,
};

/// Stencila command line tool
///
/// Enter interactive mode by using the `interact` command, the `--interact` option with
/// any other command (will be set as 'prefix'), or not supply any command.
#[derive(Parser)]
#[clap(
    version,
    infer_subcommands = true,
    global_setting = AppSettings::DeriveDisplayOrder
)]
pub struct Cli {
    /// The command to run
    #[clap(subcommand)]
    pub command: Command,

    /// Format to display results of commands (e.g. json, yaml, md)
    ///
    /// If the command result can be displayed in the specified format
    /// it will be. Display format preferences can be configured.
    #[clap(flatten)]
    pub display: DisplayOptions,

    /// Enter interactive mode (with any command and options as the prefix)
    #[clap(short, long, global = true, alias = "interactive")]
    pub interact: bool,

    /// Print debug level log events and additional diagnostics
    ///
    /// Equivalent to setting `--log-level=debug` and `--log-format=detail` and
    /// overrides the both.
    #[clap(long, global = true)]
    pub debug: bool,

    /// The minimum log level to print
    #[clap(long, global = true, possible_values = LoggingLevel::VARIANTS, ignore_case = true, env = "STENCILA_LOG_LEVEL")]
    pub log_level: Option<LoggingLevel>,

    /// The format to print log events
    #[clap(long, global = true, possible_values = LoggingFormat::VARIANTS, ignore_case = true, env = "STENCILA_LOG_FORMAT")]
    pub log_format: Option<LoggingFormat>,
}

#[derive(Debug, Default, Parser)]
pub struct DisplayOptions {
    /// Format to display output values (if possible)
    #[clap(long = "as", name = "format", alias = "display", global = true, conflicts_with_all = &["json", "yaml", "md"])]
    pub display: Option<String>,

    /// Display output values as JSON (alias for `--as json`)
    #[clap(long, global = true, conflicts_with_all = &["format", "yaml", "md"])]
    pub json: bool,

    /// Display output values as YAML (alias for `--as yaml`)
    #[clap(long, global = true, conflicts_with_all = &["format", "json", "md"])]
    pub yaml: bool,

    /// Display output values as Markdown if possible (alias for `--as md`)
    #[clap(long, global = true, conflicts_with_all = &["format", "json", "yaml"])]
    pub md: bool,
}

impl DisplayOptions {
    fn to_format(&self) -> Option<String> {
        if self.display.is_some() {
            self.display.clone()
        } else if self.json {
            some_string!("json")
        } else if self.yaml {
            some_string!("yaml")
        } else if self.md {
            some_string!("md")
        } else {
            None
        }
    }
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

#[derive(Parser)]
#[clap(
    infer_subcommands = true,
    global_setting = AppSettings::DeriveDisplayOrder,
)]
pub enum Command {
    // General commands that delegate to either the `projects` module,
    // or the `documents` module (depending upon if path is a folder or file),
    // or combine results from both in the case of `List`.
    List(ListCommand),
    Open(OpenCommand),
    Close(CloseCommand),
    Show(ShowCommand),
    Run(RunCommand),
    Convert(ConvertCommand),
    Diff(DiffCommand),
    Merge(MergeCommand),

    // The special `with` command which enters interactive mode with
    // `projects <placeholder> <path>` or `documents <placeholder> <path>`
    // as the command prefix
    With(WithCommand),

    #[clap(aliases = &["document", "docs", "doc"])]
    Documents(documents::commands::Command),

    #[clap(aliases = &["project"])]
    Projects(cloud::projects::cli::Command),

    #[clap(aliases = &["source"])]
    Sources(cloud::sources::cli::Command),

    #[clap(aliases = &["tasks"])]
    Tasks(tasks::cli::Command),

    Orgs(cloud::orgs::cli::Command),
    Teams(cloud::teams::cli::Command),
    Users(cloud::users::cli::Command),

    #[cfg(feature = "codecs-cli")]
    #[clap(aliases = &["codec"])]
    Codecs(codecs::commands::Command),

    #[cfg(feature = "parsers-cli")]
    #[clap(aliases = &["parser"])]
    Parsers(parsers::commands::Command),

    #[cfg(feature = "kernels-cli")]
    #[clap(aliases = &["kernel"])]
    Kernels(kernels::commands::Command),

    #[cfg(feature = "binaries-cli")]
    #[clap(aliases = &["binary"])]
    Binaries(binaries::commands::Command),

    #[cfg(feature = "providers-cli")]
    #[clap(aliases = &["provider"])]
    Providers(providers::commands::Command),

    #[cfg(feature = "images-cli")]
    #[clap(aliases = &["image"])]
    Images(images::cli::Command),

    #[cfg(feature = "plugins-cli")]
    #[clap(aliases = &["plugin"])]
    Plugins(plugins::commands::Command),

    #[cfg(feature = "server")]
    #[clap(aliases = &["server"])]
    Server(crate::server::commands::Command),

    Config(config::commands::Command),

    Login(cloud::users::cli::Login),
    Logout(cloud::users::cli::Logout),
    Tokens(cloud::tokens::cli::Command),

    #[cfg(feature = "upgrade")]
    Upgrade(crate::upgrade::commands::Command),

    /// Enter interactive mode (if not yet in it)
    Interact,
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match self {
            Command::List(command) => command.run().await,
            Command::Open(command) => command.run().await,
            Command::Close(command) => command.run().await,
            Command::Show(command) => command.run().await,
            Command::Run(command) => command.run().await,
            Command::Convert(command) => command.run().await,
            Command::Diff(command) => command.run().await,
            Command::Merge(command) => command.run().await,
            Command::With(command) => command.run().await,
            Command::Documents(command) => command.run().await,

            Command::Projects(command) => command.run().await,
            Command::Sources(command) => command.run().await,
            Command::Orgs(command) => command.run().await,
            Command::Teams(command) => command.run().await,
            Command::Users(command) => command.run().await,

            #[cfg(feature = "tasks-cli")]
            Command::Tasks(command) => command.run().await,

            #[cfg(feature = "codecs-cli")]
            Command::Codecs(command) => command.run().await,

            #[cfg(feature = "parsers-cli")]
            Command::Parsers(command) => command.run().await,

            #[cfg(feature = "kernels-cli")]
            Command::Kernels(command) => command.run().await,

            #[cfg(feature = "binaries-cli")]
            Command::Binaries(command) => command.run().await,

            #[cfg(feature = "providers-cli")]
            Command::Providers(command) => command.run().await,

            #[cfg(feature = "buildpacks-cli")]
            Command::Buildpacks(command) => command.run().await,

            #[cfg(feature = "images-cli")]
            Command::Images(command) => command.run().await,

            #[cfg(feature = "plugins-cli")]
            Command::Plugins(command) => command.run().await,

            #[cfg(feature = "server")]
            Command::Server(command) => command.run().await,

            Command::Config(command) => command.run().await,

            Command::Login(command) => command.run().await,
            Command::Logout(command) => command.run().await,
            Command::Tokens(command) => command.run().await,

            #[cfg(feature = "upgrade")]
            Command::Upgrade(command) => command.run().await,

            Command::Interact => result::nothing(),
        }
    }
}

// The clap args used in interactive mode
#[derive(Parser)]
#[clap(no_binary_name = true)]
pub struct Line {
    #[clap(subcommand)]
    pub command: Command,

    #[clap(flatten)]
    pub display: DisplayOptions,
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
    async fn print(&self, formats: &[String], error_format: &str) {
        match self.run().await {
            Ok(value) => {
                let mut formats: Vec<String> = formats.into();
                if let Some(format) = self.display.to_format() {
                    formats.insert(0, format)
                }
                if let Err(error) = result::print::value(value, &formats) {
                    result::print::error(error, error_format)
                }
            }
            Err(error) => result::print::error(error, error_format),
        }
    }
}

/// List all open project and documents.
#[derive(Parser)]
pub struct ListCommand;

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
#[derive(Parser)]
pub struct OpenCommand {
    /// The file or folder to open
    path: Option<PathBuf>,
}

#[async_trait]
impl Run for OpenCommand {
    #[cfg(feature = "server")]
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
            let url = crate::server::serve(&path, Some(15), true).await?;
            tracing::info!("Opening in browser {}", url);
            webbrowser::open(&url)?;
        } else {
            // Provide the user with a URL containing a longer-expiry, single-use token.
            let url = crate::server::serve(&path, Some(3600), true).await?;
            tracing::info!("Available at {}", url)
        }

        // If not in interactive mode then just sleep here forever to avoid finishing
        if std::env::var("STENCILA_INTERACT_MODE").is_err() {
            use tokio::time::{sleep, Duration};
            sleep(Duration::MAX).await;
        }

        result::nothing()
    }

    #[cfg(not(feature = "server"))]
    async fn run(&self) -> Result {
        eyre::bail!("The `server` feature has not been enabled")
    }
}

/// Close a project or document
///
/// If the path is a file, the associated open document (if any) will be closed.
/// If the path is a folder, the associated project (if any) will be closed.
/// Closing a document or project just means that it is unloaded from memory
/// and the file or folder is not longer watched for changes.
#[derive(Parser)]
pub struct CloseCommand {
    /// The file or folder to close
    #[clap(default_value = ".")]
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
#[derive(Parser)]
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

/// Currently, these top-level commands simply delegate to those in other modules
type ConvertCommand = codecs::commands::Convert;
type DiffCommand = documents::commands::Diff;
type MergeCommand = documents::commands::Merge;

/// Run commands interactively with a particular project or document
#[derive(Parser)]
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

/// Run documents, tasks, and/or server
///
/// Use this command to quickly run one or more documents, tasks or the server.
/// It provides a short cut to the `documents run`, `tasks run`, and `server run`
/// subcommands and allows you to chain those together.
///
/// ## Tasks
///
/// Given a `Taskfile.yaml` in the current directory with a task named `simulation`,
/// the command,
///
/// ```sh
/// stencila run simulation n=100
/// ```
///
/// is equivalent to `stencila tasks run simulation n=100`.
///
/// All tasks in the `Taskfile.yaml` with a `schedule` or `watches` can be
/// run concurrently using,
///
/// ```sh
/// stencila run tasks
/// ```
///
/// which is equivalent to `stencila tasks run`.
///
/// ## Documents
///
/// If the current directory does not have a `Taskfile.yaml`, or the argument does not
/// match a task in the current Taskfile, the argument will be assumed to be a filename.
///
/// The command,
///
/// ```sh
/// stencila run report.md
/// ```
///
/// is equivalent to `stencila documents run report.md`.
///
/// ## Server
///
/// The argument `server` will run the server with default options e.g.
///
/// ```sh
/// stencila run server
/// ```
///
/// is equivalent to `stencila server run`.
///
/// ## Backgrounding
///
/// Things can be run in the background by adding a tilde `~`. For example, to run a task
/// and a document concurrently,
///
/// ```sh
/// stencila run simulation~ n=100 report.md~
/// ```
///
/// ## Default
///
/// If no arguments are supplied, the default is to run all tasks with a `schedule` or `watches`
/// in the background (if a `Taskfile.yaml` is present), and to run the server i.e.
///
/// ```sh
/// stencila run
/// ```
///
/// is equivalent to `stencila run tasks~ server`.
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct RunCommand {
    /// Run arguments
    args: Vec<String>,
}

#[async_trait]
impl Run for RunCommand {
    async fn run(&self) -> Result {
        let taskfile = PathBuf::from("Taskfile.yaml");
        let taskfile = match taskfile.exists() {
            true => Some(Taskfile::read(&taskfile, 2)?),
            false => None,
        };

        let args = if self.args.is_empty() {
            let mut args = Vec::new();
            if taskfile.is_some() {
                args.push("tasks~".to_string());
            }
            args.push("server".to_string());
            args
        } else {
            self.args.clone()
        };

        let mut module = String::new();
        let mut module_args = Vec::new();
        let mut background = false;
        for index in 0..args.len() {
            let arg = &args[index];

            if arg.contains('=') {
                module_args.push(arg.clone());
            } else {
                let (name, back) = match arg.strip_suffix('~') {
                    Some(name) => (name.to_string(), true),
                    None => (arg.to_string(), false),
                };

                module = if name == "server" || name == "tasks" {
                    name
                } else if taskfile
                    .as_ref()
                    .map(|taskfile| taskfile.tasks.contains_key(&name))
                    .unwrap_or(false)
                {
                    module_args.insert(0, name);
                    "tasks".to_string()
                } else {
                    module_args.insert(0, name);
                    "documents".to_string()
                };

                background = back;
            }

            if let Some(next) = args.get(index + 1) {
                if next.contains('=') {
                    continue;
                }
            }

            let mut args = module_args.clone();
            args.insert(0, "run".to_string());

            // Macro to avoid boxing and associated compiler warning with dealing with different
            // commands in the following match
            macro_rules! run {
                ($cmd: expr) => {
                    if background {
                        tracing::debug!("Running `{} {}` in background", module, args.join(" "));
                        tokio::spawn(async move { $cmd.run().await });
                    } else {
                        tracing::debug!("Running `{} {}`", module, args.join(" "));
                        $cmd.run().await?;
                    }
                };
            }

            match module.as_str() {
                "tasks" => run!(tasks::cli::Run_::try_parse_from(&args)?),
                "documents" => run!(documents::commands::Runn::try_parse_from(&args)?),
                "server" => run!(crate::server::commands::Start::try_parse_from(&args)?),
                _ => bail!("Unhandled module: {}", module),
            };

            module_args.clear();
        }

        result::nothing()
    }
}

/// Main CLI entry point function
pub async fn main() -> eyre::Result<()> {
    #[cfg(all(windows, feature = "cli-pretty"))]
    #[allow(unused_must_use)]
    {
        cli_utils::ansi_term::enable_ansi_support();
    }

    let args: Vec<String> = std::env::args().collect();

    // Parse args into a command.
    // We extend the normal clap parsing by falling back to `tasks run` if there was at least one
    // arg by they are unrecognized and to interactive mode if there are no arguments at all.
    let parsed_args = Cli::try_parse_from(args.clone());
    let Cli {
        command,
        display,
        debug,
        log_level,
        log_format,
        mut interact,
        ..
    } = match parsed_args {
        Ok(cmd) => cmd,
        Err(error) => {
            if matches!(
                error.kind(),
                clap::ErrorKind::InvalidSubcommand | clap::ErrorKind::UnrecognizedSubcommand
            ) {
                // Count the number of tasks (args that are not options)?
                // The minus one accounts for the leading binary name
                let tasks = args.iter().filter(|arg| !arg.starts_with('-')).count() - 1;

                // Inserting args and re-parsing like the following may seem a bit hacky but it
                // allows for args such as --debug etc to be captured
                if tasks == 0 {
                    // If no tasks, then go into interactive mode
                    let mut args = args.clone();
                    args.insert(1, "interact".to_string());
                    Cli::parse_from(args)
                } else {
                    // Otherwise print claps message which for `InvalidSubcommand`
                    // will be a suggestion
                    error.print()?;
                    std::process::exit(exitcode::USAGE);
                }
            } else {
                // Note that this branch includes `clap::ErrorKind::DisplayHelp` (--help) and
                // `clap::ErrorKind::DisplayVersion` (--version)
                error.print()?;
                std::process::exit(match error.kind() {
                    clap::ErrorKind::DisplayHelp | clap::ErrorKind::DisplayVersion => exitcode::OK,
                    _ => exitcode::USAGE,
                });
            }
        }
    };

    if matches!(command, Command::Interact) {
        interact = true
    }

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
                log_format.unwrap_or_else(|| {
                    if !stderr_isatty() {
                        LoggingFormat::Json
                    } else {
                        logging_config.stderr.format
                    }
                })
            },
        },
        ..logging_config
    };

    // To ensure all log events get written to file, take guards here, so that
    // non blocking writers do not get dropped until the end of this function.
    // See https://tracing.rs/tracing_appender/non_blocking/struct.workerguard
    let _logging_guards = logging::init(true, false, true, &logging_config)?;

    // Set up error reporting and progress indicators for better feedback to user
    #[cfg(feature = "cli-pretty")]
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
        use events::{subscribe, Subscriber};
        subscribe(
            "progress",
            Subscriber::Function(cli_utils::progress::subscriber),
        )?;
    }

    // If not explicitly upgrading then run an upgrade check in the background
    #[cfg(feature = "upgrade")]
    let upgrade_thread = {
        if let Command::Upgrade(_) = command {
            None
        } else {
            Some(crate::upgrade::upgrade_auto())
        }
    };

    // Use the desired display format, falling back to configured values
    let formats = match display.to_format() {
        Some(display) => vec![display],
        None => vec!["md".to_string(), "yaml".to_string(), "json".to_string()],
    };

    // The `with` command is always interactive; need to work out
    // if projects or documents module
    #[allow(unused_variables)]
    let (interact, module) = match &command {
        Command::With(WithCommand { path }) => (
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
    if !interact {
        let error_format =
            if matches!(logging_config.stderr.format, LoggingFormat::Json) || !stderr_isatty() {
                "json"
            } else {
                ""
            };
        command.print(&formats, error_format).await;
    } else {
        #[cfg(feature = "cli-interact")]
        {
            let mut prefix: Vec<String> = args
                .into_iter()
                // Remove executable name
                .skip(1)
                // Remove the 'interact' command and any global args which can not be applied
                // to each interactive line
                .filter(|arg| arg != "interact" && !GLOBAL_ARGS.contains(&arg.as_str()))
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

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
