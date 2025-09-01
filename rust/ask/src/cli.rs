use std::{
    env::var_os,
    io::{IsTerminal, Write, stderr, stdin},
};

use async_trait::async_trait;
use eyre::{Result, bail};
use owo_colors::OwoColorize;
use rpassword::read_password;
use textwrap::{termwidth, wrap};

use crate::{Answer, Ask, AskLevel, AskOptions};

/// CLI provider
pub struct CliProvider;

#[allow(clippy::print_stderr)]
#[async_trait]
impl Ask for CliProvider {
    async fn ask(&self, question: &str, options: AskOptions) -> Result<Answer> {
        let question = if var_os("NO_COLOR").is_none() {
            // Color content between backticks cyan when NO_COLOR is not set
            let mut result = String::new();
            let mut chars = question.chars().peekable();

            while let Some(ch) = chars.next() {
                if ch == '`' {
                    // Found opening backtick, look for closing backtick
                    let mut code_content = String::new();
                    let mut found_closing = false;

                    for inner_ch in chars.by_ref() {
                        if inner_ch == '`' {
                            found_closing = true;
                            break;
                        }
                        code_content.push(inner_ch);
                    }

                    if found_closing && !code_content.is_empty() {
                        // Add orange coloring around the content
                        result.push_str(&format!("\x1b[38;5;208m{}\x1b[0m", code_content));
                    } else {
                        // No closing backtick found or empty content, keep original
                        result.push('`');
                        result.push_str(&code_content);
                    }
                } else {
                    result.push(ch);
                }
            }
            result
        } else {
            question.to_string()
        };

        let yes = match options.default {
            Some(Answer::Yes) => format!(
                "{}{}",
                "y".green().bold().underline(),
                "es".green().underline()
            ),
            _ => format!("{}{}", "y".green().bold(), "es".green()),
        };

        let no = match options.default {
            Some(Answer::No) => {
                format!("{}{}", "n".red().bold().underline(), "o".red().underline())
            }
            _ => format!("{}{}", "n".red().bold(), "o".red()),
        };

        let cancel = match options.default {
            Some(Answer::Cancel) => format!(
                "{}{}",
                "c".blue().bold().underline(),
                "ancel".blue().underline()
            ),
            _ => format!("{}{}", "c".blue().bold(), "ancel".blue()),
        };

        let prompt = if options.cancel_enabled() {
            format!("{question} [{yes}/{no}/{cancel}]: ")
        } else {
            format!("{question} [{yes}/{no}]: ")
        };

        let initial_indent = match options.level {
            AskLevel::Info => "❔ ",
            AskLevel::Warning => "⚠️  ",
            AskLevel::Error => "🟥 ",
        };

        let width = termwidth().min(120); // Cap at 120 for readability
        let prompt = wrap(
            &prompt,
            textwrap::Options::new(width)
                .initial_indent(initial_indent)
                .subsequent_indent("   "),
        )
        .join("\n");

        // Blank line to separate from logs or other questions
        eprintln!();

        eprint!("{prompt}");
        stderr().flush()?;

        // If stdin is not a TTY and we have got here (because neither --yes or
        // ASSUME_YES=true) then bail because otherwise we'll wait forever
        if !stdin().is_terminal() {
            bail!(
                "Non-interactive environment detected. Use `--yes`, `--no` or `--cancel` option or environment variable equivalent (ASSUME_YES etc)."
            );
        }

        let mut answer = String::new();
        stdin().read_line(&mut answer)?;
        let answer = answer.trim().to_lowercase();

        // Blank line to separate from logs or other questions
        eprintln!();

        if answer.is_empty()
            && let Some(default) = options.default
        {
            return Ok(default);
        }

        Ok(match answer.as_str() {
            "yes" | "y" => Answer::Yes,
            "no" | "n" => Answer::No,
            _ if options.cancel_allowed => Answer::Cancel,
            _ => options.default.unwrap_or(Answer::No),
        })
    }

    async fn password(&self, prompt: &str) -> Result<String> {
        // Blank line to separate from logs or other questions
        eprintln!();

        let prompt = format!("{prompt}: ");
        let width = termwidth().min(120); // Cap at 120 for readability
        let prompt = wrap(
            &prompt,
            textwrap::Options::new(width)
                .initial_indent("🔒 ")
                .subsequent_indent("   "),
        )
        .join("\n");

        eprint!("{prompt}");
        stderr().flush()?;

        // If stdin is not a TTY then bail because otherwise we'll wait forever
        if !stdin().is_terminal() {
            bail!(
                "Non-interactive environment detected. Password input requires an interactive terminal."
            );
        }

        let password = read_password()?;

        // Blank line to separate from logs or other questions
        eprintln!();

        Ok(password)
    }
}
