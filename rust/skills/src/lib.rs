use std::path::{Path, PathBuf};

use eyre::{OptionExt, Result, bail};
use glob::glob;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};
use tokio::fs::read_to_string;

use stencila_codecs::{DecodeOptions, Format};
use stencila_schema::{Node, NodeType, Skill};

#[cfg(feature = "cli")]
pub mod cli;
mod validate;
mod xml;

pub use validate::{ValidationError, validate_name, validate_skill};
pub use xml::{metadata_to_xml, to_xml};

/// An instance of a skill loaded from disk
///
/// Wraps a [`Skill`] with its file path and home directory.
#[derive(Default, Clone, Deserialize)]
#[serde(default)]
pub struct SkillInstance {
    #[serde(flatten)]
    pub inner: Skill,

    /// Path to the SKILL.md file
    path: PathBuf,

    /// Home directory of the skill (parent of SKILL.md)
    #[serde(skip)]
    home: PathBuf,
}

impl std::ops::Deref for SkillInstance {
    type Target = Skill;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for SkillInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Custom serialization for display purposes
impl Serialize for SkillInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SkillInstance", 5)?;

        state.serialize_field("name", &self.inner.name)?;
        state.serialize_field("description", &self.inner.description)?;
        state.serialize_field("licenses", &self.inner.options.licenses)?;
        state.serialize_field("compatibility", &self.inner.compatibility)?;
        state.serialize_field("path", &self.path)?;

        state.end()
    }
}

impl SkillInstance {
    fn new(inner: Skill, path: PathBuf) -> Result<Self> {
        let path = path.canonicalize()?;

        let home = path
            .parent()
            .ok_or_eyre("SKILL.md not in a directory")?
            .to_path_buf();

        Ok(Self { inner, path, home })
    }

    /// Get the path to the SKILL.md file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the home directory of the skill
    pub fn home(&self) -> &Path {
        &self.home
    }
}

/// Get the closest `.stencila/skills` directory, optionally creating it
pub async fn closest_skills_dir(cwd: &Path, ensure: bool) -> Result<PathBuf> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(cwd, ensure).await?;
    stencila_dirs::stencila_skills_dir(&stencila_dir, ensure).await
}

/// List all skills in the workspace closest to the current directory
///
/// Looks for `SKILL.md` files in the closest `.stencila/skills/` directory.
pub async fn list_current() -> Vec<SkillInstance> {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            tracing::error!("Unable to get current directory: {error}");
            return Vec::new();
        }
    };

    let dir = match closest_skills_dir(&cwd, false).await {
        Ok(dir) => dir,
        Err(error) => {
            tracing::debug!("No skills directory found: {error}");
            return Vec::new();
        }
    };

    list(&dir).await
}

/// List all skills found in a skills directory
pub async fn list(skills_dir: &Path) -> Vec<SkillInstance> {
    if !skills_dir.exists() {
        return Vec::new();
    }

    match list_dir(skills_dir).await {
        Ok(skills) => skills,
        Err(error) => {
            tracing::error!(
                "While listing skills in `{}`: {error}",
                skills_dir.display()
            );
            Vec::new()
        }
    }
}

/// Get a skill by name from a skills directory
pub async fn get(skills_dir: &Path, name: &str) -> Result<SkillInstance> {
    list(skills_dir)
        .await
        .into_iter()
        .find(|skill| skill.name == name)
        .ok_or_else(|| eyre::eyre!("Unable to find skill with name `{name}`"))
}

/// List skills in a directory
///
/// Globs for `*/SKILL.md` files (one level deep), decodes each as a Skill.
async fn list_dir(skills_dir: &Path) -> Result<Vec<SkillInstance>> {
    tracing::trace!("Attempting to read skills from `{}`", skills_dir.display());

    let mut skills = vec![];
    for path in glob(&format!("{}/*/SKILL.md", skills_dir.display()))?.flatten() {
        match load_skill(&path).await {
            Ok(instance) => skills.push(instance),
            Err(error) => {
                tracing::warn!("Skipping `{}`: {error}", path.display());
            }
        }
    }

    Ok(skills)
}

/// Load a single skill from a SKILL.md path
async fn load_skill(path: &Path) -> Result<SkillInstance> {
    let content = read_to_string(path).await?;

    let node = stencila_codecs::from_str(
        &content,
        Some(DecodeOptions {
            format: Some(Format::Markdown),
            node_type: Some(NodeType::Skill),
            ..Default::default()
        }),
    )
    .await?;

    if let Node::Skill(skill) = node {
        SkillInstance::new(skill, path.to_path_buf())
    } else {
        bail!(
            "Expected `{}` to be a `Skill`, got a `{}`",
            path.display(),
            node.to_string()
        )
    }
}
