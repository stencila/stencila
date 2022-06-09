//! Functions for an interactive mode command line (REPL)

use std::path::Path;

use rustyline::error::ReadlineError;
use structopt::StructOpt;

use common::{
    eyre::{bail, eyre, Result},
    tracing,
};

use crate::command::Run;

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
Interactive mode allows you to interact with the Stencila CLI
without having to restart it. This is particularly useful for
doing things like exploring the structure of documents using `query`,
or running code within them using `execute`.

Interactive mode has the concept of a command prefix to save you having
to retype the same command and its options. For example, to interactively
query the structure of a Markdown document:

    stencila query report.Rmd --interact

You can also print, set and clear the command prefix during the
interactive session (see the shortcut keystrokes below).

"#;

    help += &Yellow.paint("SHORTCUTS:\n").to_string();
    for (keys, desc) in &[
        ("--help", "Get help for the current command prefix"),
        ("^     ", "Print the current command prefix"),
        (">     ", "Append arguments to the command prefix"),
        ("<     ", "Remove the last argument from the command prefix"),
        (">>    ", "Set the command prefix"),
        ("<<    ", "Clear the command prefix"),
        ("$     ", "Ignore the command prefix for this command"),
        ("↑     ", "Go back through command history"),
        ("↓     ", "Go forward through command history"),
        ("?     ", "Print this message"),
        ("Ctrl+C", "Cancel the current task (if any)"),
        ("Ctrl+D", "Exit interactive session"),
    ] {
        help += &format!("    {} {}\n", Green.paint(*keys), desc)
    }

    help
}

/// Run the interactive REPL
#[tracing::instrument]
pub async fn run<T>(mut prefix: Vec<String>, formats: &[String], history: &Path) -> Result<()>
where
    T: StructOpt + Run + Send + Sync,
{
    let mut rl = editor::new();
    if rl.load_history(history).is_err() {
        tracing::debug!("History file not found")
    }

    println!("{}", help());

    if !prefix.is_empty() {
        println!("Starting command prefix is {:?}", prefix);
    }

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);

                let line = line.trim();
                let mut args = line
                    .split_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<String>>();

                // Handle prefix inspection / manipulation shortcuts
                if line.starts_with('^') {
                    tracing::info!("Command prefix is: `{}`", prefix.join(" "));
                    continue;
                } else if line.starts_with(">>") {
                    prefix = args[1..].into();
                    tracing::info!("Command prefix was set to: `{}`", prefix.join(" "));
                    continue;
                } else if line.starts_with('>') {
                    prefix = [prefix, args[1..].into()].concat();
                    tracing::info!("Command prefix was appended to: `{}`", prefix.join(" "));
                    continue;
                } else if line.starts_with("<<") {
                    prefix.clear();
                    tracing::info!("Command prefix was cleared");
                    continue;
                } else if line.starts_with('<') {
                    prefix.truncate(std::cmp::max(1, prefix.len()) - 1);
                    tracing::info!("Command prefix was truncated to: `{}`", prefix.join(" "));
                    continue;
                } else if line.starts_with('?') {
                    tracing::info!("{}", help());
                    continue;
                }

                // Construct args vector for this line, handling bypassing the prefix and
                // reordering (and errors) if using the `with` command.
                let mut args = if line.starts_with('$') {
                    args.remove(0);
                    args
                } else {
                    [prefix.as_slice(), args.as_slice()].concat()
                };
                if args.len() > 1 && args[1] == "with" {
                    if args.len() == 2 {
                        tracing::error!("Using the `with` command without a path; use `>` to append one to the command prefix.");
                        continue;
                    } else if args.len() == 3 {
                        tracing::error!(
                            "Using the `with` command without a subcommand e.g `show`."
                        );
                        continue;
                    } else if args.len() > 3 {
                        let subcommand = args.remove(3);
                        args[1] = subcommand;
                    }
                };

                // Parse args and run the command
                match T::clap().get_matches_from_safe(args) {
                    Ok(matches) => {
                        let command = T::from_clap(&matches);
                        command.print(formats, "").await
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
                            eprintln!("{:?}", eyre!(error))
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                tracing::info!(
                    "Ctrl+C pressed, but no active task (use Ctrl+D to end interactive session)"
                );
            }
            Err(ReadlineError::Eof) => {
                tracing::info!("Ctrl+D pressed, ending interactive session");
                break;
            }
            Err(error) => bail!(error),
        }
    }

    rl.save_history(history)?;

    Ok(())
}

/// Module for interactive mode line editor
///
/// Implements traits for `rustyline`
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
