use std::env;

use cli_utils::{
    table::{self, Attribute, Cell, CellAlignment, Color},
    ToStdout,
};
use common::{
    derive_more::Deref, eyre::Result, once_cell::sync::Lazy, serde::Serialize,
    serde_with::skip_serializing_none,
};

pub mod cli;

/// A category of secret
#[derive(Clone, Serialize)]
#[serde(crate = "common::serde")]
enum SecretCategory {
    AiApiKey,
}

#[skip_serializing_none]
#[derive(Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct Secret {
    /// The category of the secret
    category: SecretCategory,

    /// The category of the secret
    name: String,

    /// The title of the secret
    title: String,

    /// A description of the purpose of the secret
    description: String,

    /// A redacted version of the secret (last five characters only) if available
    redacted: Option<String>,
}

impl Secret {
    fn new(category: SecretCategory, name: &str, title: &str, description: &str) -> Self {
        Self {
            category,
            name: name.into(),
            title: title.into(),
            description: description.into(),
            redacted: None,
        }
    }
}

/// A list of secrets
#[derive(Deref, Serialize)]
#[serde(crate = "common::serde")]
pub struct SecretList(Vec<Secret>);

impl ToStdout for SecretList {
    fn to_terminal(&self) -> impl std::fmt::Display {
        let mut table = table::new();
        table.set_header(["Name", "Value"]);
        for secret in self.iter() {
            table.add_row([
                Cell::new(&secret.name).add_attribute(Attribute::Bold),
                match &secret.redacted {
                    Some(redacted) => Cell::new(redacted).fg(Color::Green),
                    None => Cell::new(""),
                }
                .set_alignment(CellAlignment::Right),
            ]);
        }
        table
    }
}

/// A list of secrets used by Stencila
static SECRETS: Lazy<Vec<Secret>> = Lazy::new(|| {
    vec![
        Secret::new(
            SecretCategory::AiApiKey,
            "ANTHROPIC_API_KEY",
            "Anthropic API Key",
            "Used to access the Anthropic API",
        ),
        Secret::new(
            SecretCategory::AiApiKey,
            "GOOGLE_AI_API_KEY",
            "Google AI API Key",
            "Used to access the Google AI API",
        ),
        Secret::new(
            SecretCategory::AiApiKey,
            "OPENAI_API_KEY",
            "OpenAI API Key",
            "Used to access the OpenAI API",
        ),
        Secret::new(
            SecretCategory::AiApiKey,
            "MISTRAL_API_KEY",
            "Mistral API Key",
            "Used to access the Mistral API",
        ),
    ]
});

/// Create a keyring entry for the secret
fn entry(name: &str) -> Result<keyring::Entry> {
    Ok(keyring::Entry::new(name, "stencila")?)
}

/// List secrets
pub fn list() -> Result<SecretList> {
    SECRETS
        .iter()
        .map(|secret| {
            let redacted = entry(&secret.name)?.get_password().ok().map(|value| {
                [
                    "●".repeat(12),
                    value
                        .chars()
                        .rev()
                        .take(5)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>(),
                ]
                .concat()
            });
            Ok(Secret {
                redacted,
                ..secret.clone()
            })
        })
        .collect::<Result<Vec<Secret>>>()
        .map(SecretList)
}

/// Set a secret
pub fn set(name: &str, value: &str) -> Result<()> {
    Ok(entry(name)?.set_password(value)?)
}

/// Get a secret
pub fn get(name: &str) -> Result<String> {
    Ok(entry(name)?.get_password()?)
}

/// Get an environment variable or secret
pub fn env_or_get(name: &str) -> Result<String> {
    env::var(name).or_else(|_| get(name))
}

/// Delete a secret
pub fn delete(name: &str) -> Result<()> {
    Ok(entry(name)?.delete_password()?)
}
