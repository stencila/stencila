use std::io::{Write, stdin, stdout};

use common::{async_trait::async_trait, eyre::Result};
use textwrap::wrap;

use crate::{Answer, Ask, AskOptions};

/// CLI provider
pub struct CliProvider;

impl CliProvider {
    fn print_question(&self, question: &str) -> Result<()> {
        let question = wrap(
            question,
            textwrap::Options::new(100)
                .initial_indent("❔ ")
                .subsequent_indent("   "),
        )
        .join("\n");

        println!();
        print!("{}", question);
        stdout().flush()?;

        Ok(())
    }

    fn read_response(&self) -> Result<String> {
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        Ok(input.trim().to_lowercase())
    }
}

#[async_trait]
impl Ask for CliProvider {
    async fn ask(&self, question: &str) -> Result<Answer> {
        self.print_question(&format!("{} [y/n]: ", question))?;
        let response = self.read_response()?;

        Ok(match response.as_str() {
            "y" | "yes" => Answer::Yes,
            _ => Answer::No,
        })
    }

    async fn ask_with_options(&self, question: &str, options: AskOptions) -> Result<Answer> {
        let yes_text = options.yes_text.as_deref().unwrap_or("y").to_string();
        let no_text = options.no_text.as_deref().unwrap_or("N").to_string();

        let (yes_text, no_text) = match options.default {
            Some(true) => (yes_text.to_uppercase(), no_text),
            Some(false) => (yes_text, no_text.to_uppercase()),
            None => (yes_text, no_text),
        };

        self.print_question(&format!("{question} [{yes_text}/{no_text}]: "))?;
        let response = self.read_response()?;

        if response.is_empty() {
            if let Some(default) = options.default {
                return Ok(if default { Answer::Yes } else { Answer::No });
            }
        }

        Ok(match response.as_str() {
            s if s == yes_text.to_lowercase() || s == "yes" => Answer::Yes,
            s if s == no_text.to_lowercase() || s == "no" => Answer::No,
            _ if options.cancel_allowed => Answer::Cancel,
            _ => options
                .default
                .map_or(Answer::No, |d| if d { Answer::Yes } else { Answer::No }),
        })
    }
}
