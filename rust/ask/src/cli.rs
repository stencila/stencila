use std::io::{Write, stdin, stdout};

use owo_colors::OwoColorize;
use textwrap::wrap;

use common::{async_trait::async_trait, eyre::Result};

use crate::{Answer, Ask, AskOptions};

/// CLI provider
pub struct CliProvider;

#[async_trait]
impl Ask for CliProvider {
    async fn ask(&self, question: &str, options: AskOptions) -> Result<Answer> {
        let cancel_enabled =
            options.cancel_allowed || matches!(options.default, Some(Answer::Cancel));

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

        let prompt = if cancel_enabled {
            format!("{question} [{yes}/{no}/{cancel}]: ")
        } else {
            format!("{question} [{yes}/{no}]: ")
        };
        let prompt = wrap(
            &prompt,
            textwrap::Options::new(100)
                .initial_indent("❔ ")
                .subsequent_indent("   "),
        )
        .join("\n");

        // Blank line to separate from logs or other questions
        println!();

        print!("{prompt}");
        stdout().flush()?;

        let mut answer = String::new();
        stdin().read_line(&mut answer)?;
        let answer = answer.trim().to_lowercase();

        // Blank line to separate from logs or other questions
        println!();

        if answer.is_empty() {
            if let Some(default) = options.default {
                return Ok(default);
            }
        }

        Ok(match answer.as_str() {
            "yes" | "y" => Answer::Yes,
            "no" | "n" => Answer::No,
            _ if options.cancel_allowed => Answer::Cancel,
            _ => options.default.unwrap_or(Answer::No),
        })
    }
}
