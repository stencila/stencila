use stencila::{
    config,
    eyre::{bail, Error, Result},
    logging::{
        self,
        config::{LoggingConfig, LoggingStdErrConfig},
        LoggingFormat, LoggingLevel,
    },
    plugins, projects,
    regex::Regex,
    strum::VariantNames,
    tokio, tracing,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder,
    setting = structopt::clap::AppSettings::ColoredHelp,
    setting = structopt::clap::AppSettings::VersionlessSubcommands
)]
/// Stencila, in a terminal console, on your own machine
///
/// Enter interactive mode by using the `--interact` option with any command.
pub struct Args {
    /// The command to run
    #[structopt(subcommand)]
    pub command: Option<Command>,

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
pub const GLOBAL_ARGS: [&str; 5] = ["--interact", "-i", "--debug", "--log-level", "--log-format"];

#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::DeriveDisplayOrder
)]
pub enum Command {
    #[cfg(feature = "open")]
    Open(stencila::open::cli::Args),

    #[cfg(feature = "convert")]
    Convert(stencila::convert::cli::Args),

    #[cfg(feature = "serve")]
    Serve(stencila::serve::cli::Args),

    #[cfg(feature = "projects")]
    Projects(stencila::projects::cli::Command),

    #[cfg(feature = "plugins")]
    Plugins(stencila::plugins::cli::Args),

    #[cfg(feature = "config")]
    Config(stencila::config::cli::Args),

    #[cfg(feature = "upgrade")]
    Upgrade(stencila::upgrade::cli::Args),

    #[cfg(feature = "inspect")]
    Inspect(stencila::inspect::cli::Args),
}

#[tracing::instrument(skip(config, plugins))]
/// Run a command
pub async fn run_command(
    command: Command,
    projects: &mut projects::Projects,
    plugins: &mut plugins::Plugins,
    config: &mut config::Config,
) -> Result<()> {
    match command {
        #[cfg(feature = "open")]
        Command::Open(args) => stencila::open::cli::run(args).await,

        #[cfg(feature = "convert")]
        Command::Convert(args) => stencila::convert::cli::run(args),

        #[cfg(feature = "serve")]
        Command::Serve(args) => stencila::serve::cli::run(args, &config.serve).await,

        #[cfg(feature = "projects")]
        Command::Projects(command) => display::display(command.run(projects, &config.projects)?),

        #[cfg(feature = "plugins")]
        Command::Plugins(args) => stencila::plugins::cli::run(args, &config.plugins, plugins).await,

        #[cfg(feature = "config")]
        Command::Config(args) => stencila::config::cli::run(args, config),

        #[cfg(feature = "upgrade")]
        Command::Upgrade(args) => stencila::upgrade::cli::run(args, &config.upgrade, plugins).await,

        #[cfg(feature = "inspect")]
        Command::Inspect(args) => stencila::inspect::cli::run(args, plugins).await,
    }
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

/// Main entry point function
#[tokio::main]
pub async fn main() -> Result<()> {
    #[cfg(feature = "feedback")]
    {
        use ansi_term::Color::Red;
        println!("{}", Red.paint("Stencila CLI is in alpha testing.\n"));
    }

    let args: Vec<String> = std::env::args().collect();

    // Parse args into a command
    let parsed_args = Args::from_iter_safe(args.clone());
    let Args {
        command,
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

    // Load plugins
    let mut plugins = plugins::Plugins::load()?;

    // Initialize projects
    let mut projects = projects::Projects::default();

    // If not explicitly upgrading then run an upgrade check in the background
    #[cfg(feature = "upgrade")]
    let upgrade_thread = if let Some(Command::Upgrade(_)) = command {
        None
    } else {
        Some(stencila::upgrade::upgrade_auto(&config.upgrade, &plugins))
    };

    // Get the result of running the command
    let result = if let Some(command) = command {
        run_command(command, &mut projects, &mut plugins, &mut config).await
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
            interact::run(prefix, &mut projects, &mut plugins, &mut config).await
        }
        #[cfg(not(feature = "interact"))]
        {
            eprintln!("Compiled with `interact` feature disabled.");
            std::process::exit(exitcode::USAGE);
        }
    };

    // Join the upgrade thread and log any errors
    #[cfg(feature = "upgrade")]
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

#[cfg(feature = "pretty")]
mod display {
    use super::*;

    pub fn display(what: Option<(String, String)>) -> Result<()> {
        let (format, content) = match &what {
            None => return Ok(()),
            Some(pair) => pair,
        };

        match format.as_str() {
            "md" => render(format, content),
            _ => highlight(format, content),
        }

        Ok(())
    }

    //
    pub fn render(_format: &str, content: &str) {
        let skin = termimad::MadSkin::default();
        println!("{}", skin.term_text(content))
    }

    pub fn highlight(format: &str, content: &str) {
        use syntect::easy::HighlightLines;
        use syntect::highlighting::{Style, ThemeSet};
        use syntect::parsing::SyntaxSet;
        use syntect::util::as_24_bit_terminal_escaped;

        let syntaxes = SyntaxSet::load_defaults_newlines();
        let themes = ThemeSet::load_defaults();

        let syntax = syntaxes
            .find_syntax_by_extension(format)
            .unwrap_or_else(|| syntaxes.find_syntax_by_extension("txt").unwrap());

        let theme = &themes.themes["base16-eighties.dark"];

        let mut highlighter = HighlightLines::new(syntax, theme);
        for line in content.lines() {
            let ranges: Vec<(Style, &str)> = highlighter.highlight(line, &syntaxes);
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            println!("{}", escaped);
        }
    }
}

#[cfg(not(feature = "pretty"))]
mod display {
    use super::*;

    pub fn display(what: (String, String)) -> Result<()> {
        let (_format, content) = what;
        println!("{}", content);
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
    use stencila::{eyre::eyre, util};

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::NoBinaryName,
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub struct Line {
        #[structopt(subcommand)]
        pub command: Command,
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
    #[tracing::instrument(skip(config, plugins))]
    pub async fn run(
        prefix: Vec<String>,
        projects: &mut projects::Projects,
        plugins: &mut plugins::Plugins,
        config: &mut config::Config,
    ) -> Result<()> {
        let history_file = util::dirs::config(true)?.join("history.txt");

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
                            let Line { command } = Line::from_clap(&matches);
                            if let Err(error) =
                                run_command(command, projects, plugins, config).await
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
