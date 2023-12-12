use std::sync::{RwLock, RwLockReadGuard};

use common::{
    eyre::{eyre, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    serde::Serialize,
    serde_json::json,
};
use minijinja::Environment;
use rust_embed::RustEmbed;

/// Embedded library of prompt files
///
/// During development these are served directly from the folder
/// but are embedded into the binary on release builds.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/src/prompts"]
struct Library;

static ENV: Lazy<RwLock<Environment>> = Lazy::new(|| RwLock::new(Environment::new()));

/// Lazily load the prompt template environment
fn load_env() -> Result<RwLockReadGuard<'static, Environment<'static>>> {
    // In production, take a read lock on environment and if the special sentinel template
    // exists return early
    if !cfg!(debug_assertions) {
        let env = ENV
            .read()
            .map_err(|error| eyre!("Unable to read environment: {error}"))?;
        if env.get_template("__loaded__").is_ok() {
            return Ok(env);
        }
    }

    // Get a write lock on the environment and load all the templates into it
    // Note that in development, this is intentionally done on each call to this function
    {
        let mut env = ENV
            .write()
            .map_err(|error| eyre!("Unable to read environment: {error}"))?;

        if cfg!(debug_assertions) {
            env.clear_templates()
        }

        // Add all the templates in the library, erroring if there is syntax errors in them
        for (name, content) in
            Library::iter().filter_map(|name| Library::get(&name).map(|file| (name, file.data)))
        {
            env.add_template_owned(name, String::from_utf8_lossy(&content).to_string())?;
        }

        // Add the specially named sentinel template to indicate that the environment is loaded
        env.add_template_owned("__loaded__", "yes")?;
    }

    // Finally, return a read lock on the environment
    ENV.read()
        .map_err(|error| eyre!("Unable to read environment: {error}"))
}

pub struct Prompt {
    /// The name of the prompt
    ///
    /// This is the path of the prompt file within the `prompts` folder
    /// e.g. `prose/discussion.txt`.
    name: String,
}

impl Prompt {
    /// Load a prompt from the library
    pub fn load(name: &str) -> Result<Self> {
        // Ensure that the templates in the library are all syntactically valid
        // and that this one exists
        let env = load_env()?;
        env.get_template(name)?;

        Ok(Self { name: name.into() })
    }

    /// Load a prompt, or if none, then the default prompt
    pub fn load_or_default(name: &Option<String>) -> Result<Self> {
        let name = name.as_ref().map_or("default.md", |name| name.as_str());
        Self::load(name)
    }

    /// Render the prompt
    ///
    /// Returns the rendered system and user prompts
    pub fn render(&self) -> Result<(String, String)> {
        self.render_with(json!({}))
    }

    /// Render the prompt with some context data
    pub fn render_with<S>(&self, data: S) -> Result<(String, String)>
    where
        S: Serialize,
    {
        let env = load_env()?;
        let template = env.get_template(&self.name)?;

        let content = template.render(data)?;
        let (.., system_prompt, user_prompt) = Prompt::split(&content)?;

        Ok((system_prompt, user_prompt))
    }

    /// Split a string into the three parts of a prompt: description, system prompt and user prompt
    fn split(content: &str) -> Result<(String, String, String)> {
        content
            .splitn(3, "***")
            .map(|part| part.trim().to_string())
            .collect_tuple()
            .ok_or_else(|| eyre!("Content does not have at least two *** separators"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompts_in_library_are_valid() -> Result<()> {
        drop(load_env()?);
        Ok(())
    }
}
