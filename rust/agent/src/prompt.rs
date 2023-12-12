use std::{
    fs::read_dir,
    path::PathBuf,
    sync::{RwLock, RwLockReadGuard},
};

use common::{
    eyre::{eyre, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    serde::Serialize,
    serde_json::json,
};
use minijinja::Environment;
use rust_embed::RustEmbed;

/// Builtin prompts
///
/// During development these are served directly from the `/prompts/builtin`
/// directory at the root of the repository but are embedded into the binary on release builds.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../prompts/builtin"]
struct Builtin;

/// The template environment populated with prompts
///
/// Needs to be  RWLock because gets mutated in development.
/// If this causes perf or other issues, could be refactored to
/// avoid RWLock for release builds.
static ENV: Lazy<RwLock<Environment>> =
    Lazy::new(|| RwLock::new(new_env().expect("unable to create prompt environment")));

/// Create a new environment with all prompts loaded into it
fn new_env() -> Result<Environment<'static>> {
    let mut env = Environment::new();

    // Add all builtin prompts, erroring if there is syntax errors in them
    for (file_name, content) in
        Builtin::iter().filter_map(|name| Builtin::get(&name).map(|file| (name, file.data)))
    {
        let path = PathBuf::from(file_name.to_string());
        let Some(name, ..) = path
                .file_stem() else {
                    continue;
                };

        let name = name.to_string_lossy().to_string();
        let source = String::from_utf8_lossy(&content).to_string();

        env.add_template_owned(name, source)?;
    }

    // If in development, also load all test prompts
    #[cfg(debug_assertions)]
    {
        let tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../prompts/test");
        for file in read_dir(tests)?.flatten() {
            let path = file.path();
            let Some(name, ..) = path
                .file_stem() else {
                    continue;
                };

            let name = name.to_string_lossy().to_string();
            let source = std::fs::read_to_string(file.path())?;

            env.add_template_owned(name, source)?;
        }
    }

    Ok(env)
}

/// Load the prompt template environment
///
/// In development builds the environment is re-populated each time
/// the function is called (to ensure any changes to the prompt on disk are
/// reflected in a running session).
fn load_env() -> Result<RwLockReadGuard<'static, Environment<'static>>> {
    // In development, take a write lock and reload the environment
    if cfg!(debug_assertions) {
        let mut env = ENV
            .write()
            .map_err(|error| eyre!("unable to write environment: {error}"))?;
        *env = new_env()?;
    }

    // Return a read lock on the environment
    ENV.read()
        .map_err(|error| eyre!("unable to read environment: {error}"))
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
