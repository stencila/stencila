#![recursion_limit = "256"]

use std::{collections::HashMap, path::PathBuf};
use stencila::{
    cli::display,
    config, documents,
    eyre::{bail, Error, Result},
    logging::{
        self,
        config::{LoggingConfig, LoggingStdErrConfig},
        LoggingFormat, LoggingLevel,
    },
    plugins, projects,
    regex::Regex,
    serde_json, serde_yaml, serve,
    strum::VariantNames,
    tokio, tracing, upgrade,
};
use structopt::StructOpt;

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
    #[structopt(long, global = true)]
    pub display: Option<String>,

    /// Enter interactive mode (with any command and options as the prefix)
    #[structopt(short, long, global = true)]
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
pub const GLOBAL_ARGS: [&str; 5] = ["--interact", "-i", "--debug", "--log-level", "--log-format"];

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
    #[structopt(aliases = &["project"])]
    Projects(projects::cli::Command),

    // Module-specific commands defined in the `stencila` library
    #[structopt(aliases = &["document", "docs", "doc"])]
    Documents(documents::cli::Command),

    #[structopt(aliases = &["plugin"])]
    Plugins(plugins::cli::Command),

    Config(config::cli::Command),

    Upgrade(upgrade::cli::Args),
}
#[derive(Debug)]
pub struct Context {
    interactive: bool,

    serving: bool,
}

/// Run a command
#[tracing::instrument(skip(documents, projects, config))]
pub async fn run_command(
    command: Command,
    formats: &[String],
    context: &mut Context,
    documents: &mut documents::Documents,
    projects: &mut projects::Projects,
    config: &mut config::Config,
) -> Result<()> {
    let result = match command {
        Command::List(command) => command.run(projects, documents).await,
        Command::Open(command) => command.run(context, projects, documents, config).await,
        Command::Close(command) => command.run(projects, documents).await,
        Command::Show(command) => command.run(projects, documents, config).await,
        Command::Documents(command) => command.run(documents).await,
        Command::Projects(command) => command.run(projects, &config.projects),
        Command::Plugins(command) => plugins::cli::run(command, &config.plugins).await,
        Command::Config(command) => config::cli::run(command, config),
        Command::Upgrade(command) => upgrade::cli::run(command, &config.upgrade).await,
    };
    render::render(context.interactive, formats, result?)
}

/// List all open project and documents.
#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::NoBinaryName,
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
pub struct ListCommand {}

impl ListCommand {
    pub async fn run(
        self,
        projects: &mut projects::Projects,
        documents: &mut documents::Documents,
    ) -> display::Result {
        let mut value = HashMap::new();
        value.insert("projects", projects.list()?);
        value.insert("documents", documents.list().await?);
        display::value(value)
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
    #[structopt(default_value = ".")]
    path: PathBuf,

    /// The theme to use to view the document
    #[structopt(short, long)]
    theme: Option<String>,
}

impl OpenCommand {
    pub async fn run(
        self,
        context: &mut Context,
        projects: &mut projects::Projects,
        documents: &mut documents::Documents,
        config: &config::Config,
    ) -> display::Result {
        let Self { path, theme } = self;

        let path = if path.is_dir() {
            let project = projects.open(&path, &config.projects, true)?;
            match project.main_path {
                Some(path) => path,
                None => {
                    tracing::info!("Project has no main project to display");
                    return display::nothing();
                }
            }
        } else {
            let document = documents.open(&path, None).await?;
            document.path
        };

        // Assert that the path is inside the current working directory and
        // strip that prefix (the `cwd` is prefixed when serving and path traversal checked for again)
        let path = match path.strip_prefix(std::env::current_dir()?) {
            Ok(path) => path.display().to_string(),
            Err(_) => bail!("For security reasons it is only possible to open documents that are nested within the current working directory.")
        };

        // Append the theme query parameter if set
        let path = if let Some(theme) = theme {
            format!("{path}?theme={theme}", path = path, theme = theme.to_ascii_lowercase())
        } else {
            path
        };

        // Generate a key and a corresponding login URL and open browser at the login page (will
        // redirect to document page).
        let key = serve::generate_key();
        let login_url = serve::login_url(&key, Some(60), Some(path))?;
        webbrowser::open(login_url.as_str())?;

        // If not yet serving, serve in the background, or in the current thread,
        // depending upon mode.
        if !context.serving {
            context.serving = true;
            if context.interactive {
                serve::serve_background(documents, None, Some(key))?
            } else {
                serve::serve(documents, None, Some(key)).await?;
            }
        }

        display::nothing()
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

impl CloseCommand {
    pub async fn run(
        self,
        projects: &mut projects::Projects,
        documents: &mut documents::Documents,
    ) -> display::Result {
        let Self { path } = self;

        if path.is_dir() {
            projects.close(&path)?;
        } else {
            documents.close(&path).await?;
        }

        display::nothing()
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
    #[structopt(default_value = ".")]
    path: PathBuf,
}

impl ShowCommand {
    pub async fn run(
        self,
        projects: &mut projects::Projects,
        documents: &mut documents::Documents,
        config: &config::Config,
    ) -> display::Result {
        let Self { path } = self;

        if path.is_dir() {
            display::value(projects.open(&path, &config.projects, true)?)
        } else {
            display::value(documents.open(&path, None).await?)
        }
    }
}

/// Main entry point function
#[tokio::main]
pub async fn main() -> Result<()> {
    #[cfg(feature = "feedback")]
    {
        use ansi_term::Color::Red;
        eprintln!("{}", Red.paint("Stencila CLI is in alpha testing.\n"));
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
        ..
    } = match parsed_args {
        Ok(args) => args,
        Err(error) => {
            if args.contains(&"-i".to_string()) || args.contains(&"--interact".to_string()) {
                // Parse the global options ourselves so that user can
                // pass an incomplete command prefix to interactive mode
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
    // when reading the config.
    let prelim_subscriber_guard = logging::prelim();
    let mut config = config::read()?;
    drop(prelim_subscriber_guard);

    // Create a logging config with local overrides
    let logging_config = config.logging.clone();
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
    #[cfg(feature = "feedback")]
    {
        // Setup `color_eyre` crate for better error reporting with span and back traces
        if std::env::var("RUST_SPANTRACE").is_err() {
            std::env::set_var("RUST_SPANTRACE", if debug { "1" } else { "0" });
        }
        if std::env::var("RUST_BACKTRACE").is_err() {
            std::env::set_var("RUST_BACKTRACE", if debug { "full" } else { "0" });
        }
        color_eyre::config::HookBuilder::default()
            .display_env_section(false)
            .install()?;

        // Subscribe to progress events and display them on console
        stencila::pubsub::subscribe("progress", feedback::progress_subscriber)?;
    }

    // Create document store
    let mut documents = documents::Documents::new();

    // Initialize projects
    let mut projects = projects::Projects::new();

    // If not explicitly upgrading then run an upgrade check in the background
    let upgrade_thread = if let Some(Command::Upgrade(_)) = command {
        None
    } else {
        Some(stencila::upgrade::upgrade_auto(&config.upgrade))
    };

    // Use the desired display format, falling back to configured values
    let formats = match display {
        Some(display) => vec![display],
        None => vec!["md".to_string(), "yaml".to_string(), "json".to_string()],
    };

    // Create the CLI context to pass down
    let mut context = Context {
        interactive: command.is_none(),
        serving: false,
    };

    // Get the result of running the command
    let result = if let Some(command) = command {
        run_command(
            command,
            &formats,
            &mut context,
            &mut documents,
            &mut projects,
            &mut config,
        )
        .await
    } else {
        #[cfg(feature = "interact")]
        {
            let prefix: Vec<String> = args
                .into_iter()
                // Remove executable name
                .skip(1)
                // Remove the global args which can not be applied to each interactive line
                .filter(|arg| !GLOBAL_ARGS.contains(&arg.as_str()))
                .collect();
            interact::run(
                prefix,
                &formats,
                &mut context,
                &mut documents,
                &mut projects,
                &mut config,
            )
            .await
        }
        #[cfg(not(feature = "interact"))]
        {
            eprintln!("Compiled with `interact` feature disabled.");
            std::process::exit(exitcode::USAGE);
        }
    };

    // Join the upgrade thread and log any errors
    if let Some(upgrade_thread) = upgrade_thread {
        if let Err(_error) = upgrade_thread.join() {
            tracing::warn!("Error while attempting to join upgrade thread")
        }
    }

    #[cfg(feature = "feedback")]
    match result {
        Ok(_) => Ok(()),
        Err(error) => feedback::error_reporter(error),
    }

    #[cfg(not(feature = "feedback"))]
    result
}

/// Print an error
pub fn print_error(error: Error) {
    // Remove any error label already in error string
    let re = Regex::new(r"\s*error\s*:?").unwrap();
    let error = error.to_string();
    let error = if let Some(captures) = re.captures(error.as_str()) {
        error.replace(&captures[0], "").trim().into()
    } else {
        error
    };
    eprintln!("ERROR: {}", error);
}
/// Module for feedback features
///
/// These features are aimed at providing better feedback on
/// errors and progress
#[cfg(feature = "feedback")]
mod feedback {
    use std::{collections::HashMap, sync::Mutex};

    use ansi_term::Color::{Blue, Purple};
    use color_eyre::{Help, SectionExt};
    use linya::{Bar, Progress};
    use stencila::{eyre, once_cell::sync::Lazy, pubsub::ProgressEvent, serde_json};

    pub static PROGRESS: Lazy<Mutex<Progress>> = Lazy::new(|| Mutex::new(Progress::new()));

    pub static PROGRESS_BARS: Lazy<Mutex<HashMap<String, Bar>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    pub fn progress_subscriber(_topic: String, event: serde_json::Value) {
        let mut progress = PROGRESS.lock().expect("Unable to lock progress");

        let ProgressEvent {
            parent,
            id,
            message,
            current,
            expected,
            ..
        } = serde_json::from_value(event).expect("Unable to deserialize event");

        // If the event is for a tasks with no parent then prefix line with PROG,
        // otherwise indent it, so it appears below parent
        let prefix = Purple
            .bold()
            .paint(if parent.is_none() { "PROG  " } else { "      " });

        // Should we draw / update a progress bar, or just print a message
        if let (Some(current), Some(expected)) = (current, expected) {
            if let Some(id) = id {
                let mut bars = PROGRESS_BARS.lock().expect("Unable to lock progress bars");

                // Get the current bar for this id, or create a new one
                let bar = match bars.get(&id) {
                    Some(bar) => bar,
                    None => {
                        let msg = format!("{}{}", prefix, message.unwrap_or_default());

                        let bar = progress.bar(expected as usize, msg);
                        bars.insert(id.clone(), bar);
                        &bars[&id]
                    }
                };

                // Set the bar's current value
                progress.set_and_draw(bar, current as usize)
            }
        } else if let Some(message) = message {
            // Just print the message
            eprintln!("{}{}", prefix, message);
        }
    }

    pub fn error_reporter(error: eyre::Report) -> eyre::Result<()> {
        Err(error).with_section(move || {
            format!(
                "Get help at {}.\nReport bugs at {}.",
                Blue.paint("https://help.stenci.la"),
                Blue.paint("https://github.com/stencila/stencila/issues")
            )
            .header("Help:")
        })?
    }
}

/// Module for displaying command results prettily
#[cfg(feature = "pretty")]
mod render {
    use super::*;
    use stencila::{cli::display::Display, once_cell::sync::Lazy};
    use syntect::easy::HighlightLines;
    use syntect::highlighting::{Style, ThemeSet};
    use syntect::parsing::SyntaxSet;
    use syntect::util::as_24_bit_terminal_escaped;

    // Display the result of a command prettily
    pub fn render(interactive: bool, formats: &[String], display: Display) -> Result<()> {
        let Display {
            content,
            format,
            value,
        } = &display;

        // Nothing to display
        if content.is_none() && value.is_none() {
            return Ok(());
        }

        // Try to display in preferred format
        for preference in formats {
            if let (Some(content), Some(format)) = (content, format) {
                if format == preference {
                    return match format.as_str() {
                        "md" => print(&format, &content),
                        _ => highlight(interactive, &format, &content),
                    };
                }
            }
            if let Some(value) = value {
                if let Some(content) = match preference.as_str() {
                    "json" => serde_json::to_string_pretty(&value).ok(),
                    "yaml" => serde_yaml::to_string(&value)
                        .map(|yaml| yaml.trim_start_matches("---\n").to_string())
                        .ok(),
                    _ => None,
                } {
                    return highlight(interactive, &preference, &content);
                }
            }
        }

        // Fallback to displaying content if available, otherwise value as JSON.
        if let (Some(content), Some(format)) = (content, format) {
            match format.as_str() {
                "md" => return print(&format, &content),
                _ => return highlight(interactive, &format, &content),
            };
        } else if let Some(value) = value {
            let json = serde_json::to_string_pretty(&value)?;
            return highlight(interactive, "json", &json);
        }

        Ok(())
    }

    // Render Markdown to the terminal
    pub fn print(_format: &str, content: &str) -> Result<()> {
        let skin = termimad::MadSkin::default();
        println!("{}", skin.term_text(content));
        Ok(())
    }

    // Apply syntax highlighting and print to terminal
    pub fn highlight(interactive: bool, format: &str, content: &str) -> Result<()> {
        if !interactive {
            println!("{}", content);
            return Ok(());
        }

        // Loading syntaxes and themes is slow. The following lazily loads both once.
        // This is fine in interactive mode because subsequent calls of this function
        // do not need to load again. However, for normal usage it is still slow.
        // TODO: Only bake in a subset of syntaxes and themes. See the following for examples of this
        // https://github.com/ducaale/xh/blob/master/build.rs
        // https://github.com/sharkdp/bat/blob/0b44aa6f68ab967dd5d74b7e02d306f2b8388928/src/assets.rs
        static SYNTAXES: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
        static THEMES: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

        let syntax = SYNTAXES
            .find_syntax_by_extension(format)
            .unwrap_or_else(|| SYNTAXES.find_syntax_by_extension("txt").unwrap());

        let mut highlighter = HighlightLines::new(syntax, &THEMES.themes["base16-eighties.dark"]);
        for line in content.lines() {
            let ranges: Vec<(Style, &str)> = highlighter.highlight(line, &SYNTAXES);
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            println!("{}", escaped);
        }

        Ok(())
    }
}

/// Module for displaying command results plainly
#[cfg(not(feature = "pretty"))]
mod display {
    use super::*;
    use stencila::cli::display::Display;

    // Display the result of a command without prettiness
    pub fn render(_interactive: bool, _formats: &[String], display: Display) -> Result<()> {
        match display {
            Display {
                content: Some(content),
                ..
            } => println!("{}", content),
            Display {
                value: Some(value), ..
            } => println!("{}", serde_json::to_string_pretty(&value)?),
            _ => (),
        };
        Ok(())
    }
}

/// Module for interactive mode
///
/// Implements the the parsing and handling of user input when in interactive mode
#[cfg(feature = "interact")]
mod interact {
    use super::*;
    use rustyline::error::ReadlineError;
    use stencila::{config, eyre::eyre};

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
        #[structopt(long, global = true)]
        pub display: Option<String>,
    }

    fn help() -> String {
        use ansi_term::{
            Colour::{Green, Yellow},
            Style,
        };

        let mut help = Style::new()
            .bold()
            .paint("Stencila CLI interactive mode\n\n")
            .to_string();

        help += &Yellow.paint("ABOUT:").to_string();
        help += r#"
    Interactive mode allows you to interact with one or more of the CLIs
    commands without having to restart the application. It is particularly
    useful for exploring the structure of documents using `select`,
    running code within them using `execute`, and inspecting variables
    using `list` and `get`.

    Interactive mode has the concept of a command prefix to save you having
    to retype the same command and its options. For example, to interactively
    execute code within the context of a RMarkdown document:

        stencila execute report.Rmd --interact

    You can also print, set and clear the command prefix during the
    interactive session (see the shortcut keystrokes below).

"#;

        help += &Yellow.paint("SHORTCUTS:\n").to_string();
        for (keys, desc) in &[
            ("--help", "Get help for the current command prefix"),
            ("^     ", "Prints the current command prefix"),
            ("<     ", "Sets the command prefix"),
            (">     ", "Clears the command prefix"),
            ("↑     ", "Go back through command history"),
            ("↓     ", "Go forward through command history"),
            ("?     ", "Prints this message"),
            ("Ctrl+C", "Cancels the current command"),
            ("Ctrl+D", "Exits interactive application"),
        ] {
            help += &format!("    {} {}\n", Green.paint(*keys), desc)
        }

        help
    }

    /// Run the interactive REPL
    #[tracing::instrument(skip(documents, projects, config))]
    pub async fn run(
        prefix: Vec<String>,
        formats: &[String],
        context: &mut Context,
        documents: &mut documents::Documents,
        projects: &mut projects::Projects,
        config: &mut config::Config,
    ) -> Result<()> {
        let history_file = config::dir(true)?.join("history.txt");

        let mut rl = interact_editor::new();
        if rl.load_history(&history_file).is_err() {
            tracing::debug!("No previous history found")
        }

        println!("{}", help());

        let mut prefix = prefix.clone();
        if !prefix.is_empty() {
            println!("Starting command prefix is {:?}", prefix);
        }

        loop {
            let readline = rl.readline("> ");
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line);

                    let args = line
                        .split_whitespace()
                        .map(str::to_string)
                        .collect::<Vec<String>>();

                    if let Some(first) = line.trim_start().chars().next() {
                        if first == '^' {
                            println!("Command prefix is currently {:?}", prefix);
                            continue;
                        } else if first == '<' {
                            prefix = args[1..].into();
                            println!("Command prefix was set to {:?}", prefix);
                            continue;
                        } else if first == '>' {
                            prefix.clear();
                            println!("Command prefix was cleared");
                            continue;
                        } else if first == '?' {
                            println!("{}", help());
                            continue;
                        }
                    };

                    let args = [prefix.as_slice(), args.as_slice()].concat();
                    match Line::clap().get_matches_from_safe(args) {
                        Ok(matches) => {
                            let Line { command, display } = Line::from_clap(&matches);

                            // Use current display format or fallback to configured preferences
                            let formats = if let Some(display) = display {
                                vec![display]
                            } else {
                                formats.into()
                            };

                            if let Err(error) =
                                run_command(command, &formats, context, documents, projects, config)
                                    .await
                            {
                                print_error(error);
                            };
                        }
                        Err(error) => {
                            if error.kind == structopt::clap::ErrorKind::VersionDisplayed {
                                print!("{}", error)
                            } else if error.kind == structopt::clap::ErrorKind::HelpDisplayed
                                || error.kind
                                    == structopt::clap::ErrorKind::MissingArgumentOrSubcommand
                            {
                                // Remove the unnecessary command / version line at the start
                                let lines = format!("{}\n", error)
                                    .to_string()
                                    .lines()
                                    .skip(1)
                                    .map(str::to_string)
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                print!("{}", lines)
                            } else {
                                tracing::debug!("{:?}", error.kind);
                                print_error(eyre!(error))
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    tracing::info!("Ctrl-C pressed, interrupting current command");
                    // TODO
                }
                Err(ReadlineError::Eof) => {
                    tracing::info!("Ctrl-D pressed, ending session");
                    break;
                }
                Err(error) => bail!(error),
            }
        }
        rl.save_history(&history_file)?;

        Ok(())
    }
}
/// Module for interactive mode line editor
///
/// Implements traits for `rustyline`
#[cfg(feature = "interact")]
mod interact_editor {
    use ansi_term::Colour::{Blue, White, Yellow};
    use rustyline::{
        completion::{Completer, FilenameCompleter, Pair},
        config::OutputStreamType,
        highlight::{Highlighter, MatchingBracketHighlighter},
        hint::{Hinter, HistoryHinter},
        validate::{MatchingBracketValidator, Validator},
        validate::{ValidationContext, ValidationResult},
        CompletionType, Context, EditMode, Editor, Result,
    };
    use rustyline_derive::Helper;
    use std::borrow::Cow::{self, Owned};

    pub fn new() -> Editor<Helper> {
        let config = rustyline::Config::builder()
            .history_ignore_space(true)
            .max_history_size(1000)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .output_stream(OutputStreamType::Stdout)
            .build();

        let mut editor = Editor::with_config(config);

        let helper = Helper::new();
        editor.set_helper(Some(helper));

        editor
    }

    #[derive(Helper)]
    pub struct Helper {
        pub completer: FilenameCompleter,
        pub hinter: HistoryHinter,
        pub validator: MatchingBracketValidator,
        pub highlighter: MatchingBracketHighlighter,
    }

    impl Helper {
        pub fn new() -> Self {
            Helper {
                completer: FilenameCompleter::new(),
                hinter: HistoryHinter {},
                validator: MatchingBracketValidator::new(),
                highlighter: MatchingBracketHighlighter::new(),
            }
        }
    }

    /// Provides tab-completion candidates
    ///
    /// https://github.com/kkawakam/rustyline/blob/master/src/completion.rs
    impl Completer for Helper {
        type Candidate = Pair;

        fn complete(
            &self,
            line: &str,
            pos: usize,
            ctx: &Context<'_>,
        ) -> Result<(usize, Vec<Self::Candidate>)> {
            self.completer.complete(line, pos, ctx)
        }
    }

    /// Provides hints based on the current line
    ///
    /// See https://github.com/kkawakam/rustyline/blob/master/src/hint.rs
    impl Hinter for Helper {
        type Hint = String;

        // Takes the currently edited line with the cursor position and returns the string that should be
        // displayed or None if no hint is available for the text the user currently typed
        fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
            self.hinter.hint(line, pos, ctx)
        }
    }

    /// Determines whether the current buffer is a valid command or should continue.
    ///
    /// Will not validate unless brackets (round, square and curly) are balanced.
    impl Validator for Helper {
        fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
            self.validator.validate(ctx)
        }

        fn validate_while_typing(&self) -> bool {
            self.validator.validate_while_typing()
        }
    }

    /// Syntax highlighter
    ///
    /// Highlights brackets, prompt, hints and completion candidates.
    /// See https://github.com/kkawakam/rustyline/blob/master/src/highlight.rs
    impl Highlighter for Helper {
        fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
            self.highlighter.highlight(line, pos)
        }

        fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
            &'s self,
            prompt: &'p str,
            _default: bool,
        ) -> Cow<'b, str> {
            Owned(Blue.paint(prompt).to_string())
        }

        fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
            Owned(White.dimmed().paint(hint).to_string())
        }

        fn highlight_candidate<'c>(
            &self,
            candidate: &'c str,
            _completion: CompletionType,
        ) -> Cow<'c, str> {
            Owned(Yellow.dimmed().paint(candidate).to_string())
        }

        fn highlight_char(&self, line: &str, pos: usize) -> bool {
            self.highlighter.highlight_char(line, pos)
        }
    }
}
