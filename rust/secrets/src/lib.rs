use std::env;

use common::{
    eyre::{Result, bail},
    itertools::Itertools,
    once_cell::sync::Lazy,
    serde::Serialize,
    serde_with::skip_serializing_none,
    tracing,
};

pub mod cli;

/// A category of secret
#[derive(Clone, Serialize)]
#[serde(crate = "common::serde")]
enum SecretCategory {
    /// Used to access external services for creating content, esp. LLMs.
    AiApiKey,

    /// Used to access external REST APIs
    RestApiKey,

    /// Used to publish Stencila documents to an external service,
    /// and/or to update a document based on externally-hosted content.
    ReadWriteApiKey,
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

/// A list of secrets used by Stencila
static SECRETS: Lazy<Vec<Secret>> = Lazy::new(|| {
    vec![
        Secret::new(
            SecretCategory::AiApiKey,
            "STENCILA_API_TOKEN",
            "Stencila API Token",
            "Used for Stencila Cloud's model router and other services",
        ),
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
            SecretCategory::RestApiKey,
            "GITHUB_TOKEN",
            "GitHub API Token",
            "Used to access the GitHub REST API",
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
        Secret::new(
            SecretCategory::ReadWriteApiKey,
            "GHOST_ADMIN_API_KEY",
            "Ghost Admin API Key",
            "Used to read from and publish to Ghost",
        ),
    ]
});

/// Validate a name is a known secret
pub fn name_validator(name: &str) -> Result<String, String> {
    let possible_values = SECRETS
        .iter()
        .map(|secret| secret.name.as_str())
        .collect_vec();
    if possible_values.contains(&name) {
        Ok(name.to_string())
    } else {
        Err(format!(
            "not a known secret name [{}]",
            possible_values.join(", ")
        ))
    }
}

/// Create a keyring entry for the secret
fn entry(name: &str) -> Result<keyring::Entry> {
    Ok(keyring::Entry::new(name, "stencila")?)
}

/// List secrets
pub fn list() -> Result<Vec<Secret>> {
    tracing::trace!("Listing secrets");

    SECRETS
        .iter()
        .map(|secret| {
            Ok(Secret {
                redacted: entry(&secret.name)?.get_password().ok().map(redact),
                ..secret.clone()
            })
        })
        .collect::<Result<Vec<Secret>>>()
}

/// Redact a secret
///
/// Returns a string with the same number of characters as the secret but all
/// but any prefix (e.g `api-` or `gho_` up to four chars) and last three
/// characters redacted. If the suffix is less than 6 characters then all
/// characters will be redacted.
pub fn redact(value: String) -> String {
    let chars_count = value.chars().count();

    const CLEAR_CHARS_AT_END: usize = 3;
    const MAX_PREFIX_CHARS: usize = 4;

    if chars_count <= CLEAR_CHARS_AT_END * 2 {
        "●".repeat(chars_count)
    } else {
        // Find prefix length (letters followed by - or _, up to 4 chars)
        let prefix_len = value
            .chars()
            .take(MAX_PREFIX_CHARS)
            .take_while(|c| c.is_ascii_alphabetic())
            .count()
            .min(MAX_PREFIX_CHARS - 1); // Leave room for separator

        let prefix_len = if prefix_len > 0 && prefix_len < value.chars().count() {
            let separator = value.chars().nth(prefix_len);
            if separator == Some('-') || separator == Some('_') {
                prefix_len + 1 // Include the separator
            } else {
                0 // No valid prefix pattern found
            }
        } else {
            0
        };

        let prefix_len = prefix_len.min(chars_count - CLEAR_CHARS_AT_END);
        let chars: Vec<char> = value.chars().collect();

        [
            chars[0..prefix_len].iter().collect::<String>(),
            "●".repeat(chars_count - prefix_len - CLEAR_CHARS_AT_END),
            chars[chars_count - CLEAR_CHARS_AT_END..]
                .iter()
                .collect::<String>(),
        ]
        .concat()
    }
}

/// Set a secret
///
/// If the value is a blank string then delete the entry
#[tracing::instrument(skip(value))] // Important: skip logging secret value
pub fn set(name: &str, value: &str) -> Result<()> {
    tracing::trace!("Setting secret `{name}`");

    if !cfg!(test) && !SECRETS.iter().any(|secret| secret.name == name) {
        bail!("Only secrets used by Stencila can be set by Stencila")
    }

    let secret_entry = entry(name)?;
    if value.trim().is_empty() {
        secret_entry.delete_credential()?;
    } else {
        secret_entry.set_password(value)?;
    }

    Ok(())
}

/// Get a secret
#[tracing::instrument]
pub fn get(name: &str) -> Result<String> {
    tracing::trace!("Getting secret `{name}`");

    Ok(entry(name)?.get_password()?)
}

/// Get an environment variable or secret
#[tracing::instrument]
pub fn env_or_get(name: &str) -> Result<String> {
    env::var(name).or_else(|_| get(name))
}

/// Delete a secret
#[tracing::instrument]
pub fn delete(name: &str) -> Result<()> {
    tracing::trace!("Deleting secret `{name}`");

    if !cfg!(test) && !SECRETS.iter().any(|secret| secret.name == name) {
        bail!("Only secrets used by Stencila can be deleted by Stencila")
    }

    match entry(name)?.delete_credential() {
        Err(keyring::Error::NoEntry) => {
            tracing::warn!("No secret named {name} to delete");
            Ok(())
        }
        Err(error) => bail!(error),
        Ok(..) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_delete() -> Result<()> {
        #[allow(clippy::print_stderr)]
        if cfg!(target_os = "linux") && std::env::var("CI").is_ok() {
            // Requires the secrets service (e.g. gnome-keyring-daemon) to be running
            // so skip on CI where that is not usually the case.
            eprintln!("Skipping test on Linux CI");
            return Ok(());
        }

        let name = "TEST_SECRET";
        let value = "teSTSeCRET";

        set(name, value)?;
        assert_eq!(get(name)?, value);
        delete(name)?;
        assert!(get(name).is_err());

        Ok(())
    }

    #[test]
    fn test_redact() {
        // Test with prefix patterns
        assert_eq!(redact("api-1234567890".to_string()), "api-●●●●●●●890"); // 14 chars: api-(4) + ●●●●●●●(7) + 890(3)
        assert_eq!(redact("gho_abcdefghijk".to_string()), "gho_●●●●●●●●ijk"); // 15 chars: gho_(4) + ●●●●●●●●(8) + ijk(3)
        assert_eq!(redact("sk-1234567890".to_string()), "sk-●●●●●●●890"); // 13 chars: sk-(3) + ●●●●●●●(7) + 890(3)

        // Test without prefix patterns
        assert_eq!(redact("1234567890".to_string()), "●●●●●●●890"); // 10 chars: ●●●●●●●(7) + 890(3)
        assert_eq!(redact("abcdefghijk".to_string()), "●●●●●●●●ijk"); // 11 chars: ●●●●●●●●(8) + ijk(3)

        // Test short strings (all redacted)
        assert_eq!(redact("abc".to_string()), "●●●");
        assert_eq!(redact("12345".to_string()), "●●●●●");
        assert_eq!(redact("123456".to_string()), "●●●●●●");

        // Test edge cases
        assert_eq!(redact("a-123".to_string()), "●●●●●"); // Too short for prefix
        assert_eq!(redact("ab_1234567".to_string()), "ab_●●●●567"); // 10 chars: ab_(3) + ●●●●(4) + 567(3)
        assert_eq!(redact("abc-1234567".to_string()), "abc-●●●●567"); // 11 chars: abc-(4) + ●●●●(4) + 567(3)
        assert_eq!(redact("abcd_1234567".to_string()), "●●●●●●●●●567"); // 12 chars: abcd_(5) exceeds max 4, so no prefix + ●●●●●●●●●(9) + 567(3)

        // Test without valid separators
        assert_eq!(redact("abc1234567".to_string()), "●●●●●●●567"); // 10 chars: ●●●●●●●(7) + 567(3)
        assert_eq!(redact("api1234567".to_string()), "●●●●●●●567"); // 10 chars: ●●●●●●●(7) + 567(3)
    }
}
