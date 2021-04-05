use crate::{
    cli::{print_error, Command},
    config, convert, open, plugins, serve, upgrade,
    util::dirs,
};
use anyhow::{anyhow, bail, Result};
use rustyline::error::ReadlineError;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Stencila command line tool",
    setting = structopt::clap::AppSettings::NoBinaryName,
    setting = structopt::clap::AppSettings::ColoredHelp,
    setting = structopt::clap::AppSettings::VersionlessSubcommands
)]
pub struct Line {
    #[structopt(subcommand)]
    pub command: Command,
}

/// Run the interactive REPL
#[tracing::instrument]
pub async fn run(prefix: &Vec<String>, config: &config::Config) -> Result<()> {
    let history_file = dirs::config(true)?.join("history.txt");

    let mut rl = editor::new();
    if rl.load_history(&history_file).is_err() {
        tracing::debug!("No previous history found")
    }

    let mut prefix = prefix.clone();
    let mut config = config.clone();

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);

                let mut args = line
                    .split_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<String>>();

                if let Some(first) = line.trim_start().chars().nth(0) {
                    if first == '~' {
                        println!("Command prefix is {:?}", prefix);
                        continue;
                    } else if first == '<' {
                        prefix = args[1..].into();
                        println!("Set command prefix to {:?}", prefix);
                        continue;
                    } else if first == '>' {
                        prefix.clear();
                        println!("Cleared command prefix");
                        continue;
                    } else if first == '?' {
                        args[0] = "help".into();
                    }
                };

                let args = [prefix.as_slice(), args.as_slice()].concat();
                match Line::clap().get_matches_from_safe(args) {
                    Ok(matches) => {
                        let Line { command } = Line::from_clap(&matches);
                        if let Err(error) = match command {
                            Command::Open(args) => open::cli::run(args).await,
                            Command::Convert(args) => convert::cli::run(args),
                            Command::Serve(args) => serve::cli::run(args, &config.serve).await,
                            Command::Plugins(args) => {
                                plugins::cli::run(args, &config.plugins).await
                            }
                            Command::Upgrade(args) => upgrade::cli::run(args, &config.upgrade),
                            Command::Config(args) => match config::cli::run(args, &config) {
                                Ok(config_changed) => {
                                    // Update the configuration (may have been changed by `set` and `reset`)
                                    config = config_changed;
                                    Ok(())
                                }
                                Err(err) => Err(err),
                            },
                        } {
                            print_error(error)
                        }
                    }
                    Err(error) => {
                        if error.kind == structopt::clap::ErrorKind::VersionDisplayed {
                            print!("{}", error)
                        } else if error.kind == structopt::clap::ErrorKind::HelpDisplayed
                            || error.kind == structopt::clap::ErrorKind::MissingArgumentOrSubcommand
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
                            print_error(anyhow!(error))
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

mod editor {
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
