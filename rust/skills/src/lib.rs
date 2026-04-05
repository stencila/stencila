use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use derive_more::{Deref, DerefMut};
use eyre::{OptionExt, Result, bail};
use glob::glob;
use rust_embed::RustEmbed;
use serde::Serialize;
use strum::{Display, EnumIter, IntoEnumIterator};
use tokio::fs::read_to_string;

use stencila_codecs::{DecodeOptions, Format};
use stencila_schema::{Node, NodeType, Skill};
use stencila_version::STENCILA_VERSION;

#[cfg(feature = "cli")]
pub mod cli;
mod validate;
mod xml;

pub use validate::{ValidationError, validate_name, validate_skill};
pub use xml::{metadata_to_xml, to_xml};

/// Subdirectory name within each dot-directory that holds skills.
const SKILLS_SUBDIR: &str = "skills";

/// Where a skill was discovered from.
///
/// Each variant corresponds to a dot-directory that may contain a `skills/`
/// subdirectory. Sources listed later in [`SkillSource::all`] take precedence
/// when names conflict (last wins).
#[derive(Debug, Display, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, EnumIter)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SkillSource {
    /// Builtin Stencila skills
    #[default]
    Builtin,
    /// Skill from the workspace's `.stencila/skills/` folder
    Stencila,
    /// Skill from the workspace's `.claude/skills/` folder
    Claude,
    /// Skill from the workspace's `.codex/skills/` folder
    Codex,
    /// Skill from the workspace's `.gemini/skills/` folder
    Gemini,
}

impl SkillSource {
    /// Get a vector of all known skill sources
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    /// The dot-directory name for this source, if it has one.
    ///
    /// Returns `None` for `Builtin` since builtins are not loaded from a
    /// dot-directory on disk.
    fn dot_dir_name(&self) -> Option<&'static str> {
        match self {
            Self::Builtin => None,
            Self::Stencila => Some(".stencila"),
            Self::Claude => Some(".claude"),
            Self::Codex => Some(".codex"),
            Self::Gemini => Some(".gemini"),
        }
    }

    /// Sources for a given provider ID, with Stencila as the base layer.
    ///
    /// The `provider_id` matches values returned by `ProviderProfile::id()`:
    /// `"anthropic"`, `"openai"`, `"gemini"`.
    pub fn for_provider(provider_id: &str) -> Vec<Self> {
        let mut sources = vec![Self::Stencila];
        match provider_id {
            "anthropic" => sources.push(Self::Claude),
            "openai" => sources.push(Self::Codex),
            "gemini" => sources.push(Self::Gemini),
            _ => {}
        }
        sources
    }
}

/// Resolve workspace sources to `(SkillSource, PathBuf)` pairs.
///
/// Finds the closest dot-directory for each non-builtin source and returns
/// the corresponding `skills/` subdirectory path. The returned order matches
/// the input order, which determines precedence (last wins).
fn source_skill_dirs(cwd: &Path, sources: &[SkillSource]) -> Vec<(SkillSource, PathBuf)> {
    sources
        .iter()
        .filter_map(|&source| {
            let dot_dir_name = source.dot_dir_name()?;
            let dot_dir = stencila_dirs::closest_dot_dir(cwd, dot_dir_name)?;
            Some((source, dot_dir.join(SKILLS_SUBDIR)))
        })
        .collect()
}

/// An instance of a skill loaded from disk
///
/// Wraps a [`Skill`] with its file path and home directory.
#[derive(Default, Clone, Deref, DerefMut, Serialize)]
#[serde(default)]
pub struct SkillInstance {
    #[deref]
    #[deref_mut]
    #[serde(flatten)]
    pub inner: Skill,

    /// Which source this skill was loaded from
    source: SkillSource,

    /// Path to the SKILL.md file
    path: PathBuf,

    /// Home directory of the skill (parent of SKILL.md)
    #[serde(skip)]
    home: PathBuf,
}

impl SkillInstance {
    fn new(inner: Skill, source: SkillSource, path: PathBuf) -> Result<Self> {
        let path = path.canonicalize()?;

        let home = path
            .parent()
            .ok_or_eyre("SKILL.md not in a directory")?
            .to_path_buf();

        Ok(Self {
            inner,
            path,
            home,
            source,
        })
    }

    /// Get the path to the SKILL.md file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the home directory of the skill
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Which source directory this skill was loaded from
    pub fn source(&self) -> SkillSource {
        self.source
    }
}

/// Get the closest `.stencila/skills` directory, optionally creating it
#[cfg(feature = "cli")]
pub(crate) async fn closest_skills_dir(cwd: &Path, ensure: bool) -> Result<PathBuf> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(cwd, ensure).await?;
    stencila_dirs::stencila_skills_dir(&stencila_dir, ensure).await
}

/// Discover skills from multiple source directories with dedup (last wins).
///
/// Skills are discovered from (lowest to highest precedence):
/// 1. Builtin skills embedded in the binary
/// 2. Workspace and provider-specific dot-directories
///
/// When the same skill name appears in multiple sources, the later source wins.
pub async fn discover(cwd: &Path, sources: &[SkillSource]) -> Vec<SkillInstance> {
    let mut by_name: HashMap<String, SkillInstance> = HashMap::new();

    // Builtin skills first (lowest precedence)
    for skill in list_builtin().await {
        by_name.insert(skill.name.clone(), skill);
    }

    // Workspace and provider sources (overwrite builtins)
    for (source, skills_dir) in source_skill_dirs(cwd, sources) {
        for skill in list_in_dir(&skills_dir, source).await {
            by_name.insert(skill.name.clone(), skill);
        }
    }
    let mut skills: Vec<SkillInstance> = by_name.into_values().collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Find a skill by name across builtin and workspace sources (last wins).
pub async fn get_by_name(cwd: &Path, name: &str, sources: &[SkillSource]) -> Result<SkillInstance> {
    let mut found: Option<SkillInstance> = None;

    // Check builtins first (lowest precedence)
    if let Some(skill) = list_builtin().await.into_iter().find(|s| s.name == name) {
        found = Some(skill);
    }

    // Check workspace and provider sources (overwrite builtins)
    for (source, skills_dir) in source_skill_dirs(cwd, sources) {
        let path = skills_dir.join(name).join("SKILL.md");
        if let Ok(skill) = load_skill(&path, source).await {
            found = Some(skill);
        }
    }
    found.ok_or_else(|| eyre::eyre!("Unable to find skill with name `{name}`"))
}

/// Load a single skill from a SKILL.md path.
async fn load_skill(path: &Path, source: SkillSource) -> Result<SkillInstance> {
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
        SkillInstance::new(skill, source, path.to_path_buf())
    } else {
        bail!(
            "Expected `{}` to be a `Skill`, got a `{}`",
            path.display(),
            node.to_string()
        )
    }
}

/// List all skills found in a skills directory.
async fn list_in_dir(skills_dir: &Path, source: SkillSource) -> Vec<SkillInstance> {
    if !skills_dir.exists() {
        return Vec::new();
    }

    tracing::trace!("Attempting to read skills from `{}`", skills_dir.display());

    match list_dir(skills_dir, source).await {
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

/// Glob for `*/SKILL.md` files in a directory and load each one.
async fn list_dir(skills_dir: &Path, source: SkillSource) -> Result<Vec<SkillInstance>> {
    let mut skills = vec![];
    for path in glob(&format!("{}/*/SKILL.md", skills_dir.display()))?.flatten() {
        match load_skill(&path, source).await {
            Ok(instance) => skills.push(instance),
            Err(error) => {
                tracing::warn!("Skipping `{}`: {error}", path.display());
            }
        }
    }
    Ok(skills)
}

/// Builtin skills embedded from the repo's `.stencila/skills/` directory.
///
/// During development these are loaded directly from the source directory
/// but are embedded into the binary on release builds.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../.stencila/skills"]
#[exclude = "test-*"]
struct BuiltinSkills;

static BUILTIN_SKILLS: OnceLock<Vec<SkillInstance>> = OnceLock::new();

/// List the builtin skills.
///
/// Writes embedded files to a cache directory and loads them using the
/// standard `list_dir` logic so that file-based operations (e.g. loading
/// skill content from disk) work correctly.
async fn list_builtin() -> Vec<SkillInstance> {
    // In debug mode, load directly from the repo
    if cfg!(debug_assertions) {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.stencila/skills");
        return list_in_dir(&dir, SkillSource::Builtin).await;
    }

    if let Some(skills) = BUILTIN_SKILLS.get() {
        return skills.clone();
    }

    let skills = match initialize_builtin().await {
        Ok(dir) => list_in_dir(&dir, SkillSource::Builtin).await,
        Err(error) => {
            tracing::error!("While initializing builtin skills: {error}");
            Vec::new()
        }
    };

    BUILTIN_SKILLS.get_or_init(|| skills.clone()).clone()
}

/// Initialize the builtin skills directory by writing embedded files to disk.
pub async fn initialize_builtin() -> Result<PathBuf> {
    let dir = stencila_dirs::get_versioned_app_dir(
        stencila_dirs::DirType::BuiltinSkills,
        STENCILA_VERSION,
        cfg!(debug_assertions),
        true,
    )?;

    let files = BuiltinSkills::iter().filter_map(|filename| {
        BuiltinSkills::get(&filename)
            .map(|file| (PathBuf::from(filename.as_ref()), file.data.to_vec()))
    });
    stencila_dirs::ensure_embedded_dir(&dir, files)?;

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- SkillSource unit tests --

    #[test]
    fn skill_source_for_provider_anthropic() {
        let sources = SkillSource::for_provider("anthropic");
        assert_eq!(sources, vec![SkillSource::Stencila, SkillSource::Claude]);
    }

    #[test]
    fn skill_source_for_provider_openai() {
        let sources = SkillSource::for_provider("openai");
        assert_eq!(sources, vec![SkillSource::Stencila, SkillSource::Codex]);
    }

    #[test]
    fn skill_source_for_provider_gemini() {
        let sources = SkillSource::for_provider("gemini");
        assert_eq!(sources, vec![SkillSource::Stencila, SkillSource::Gemini]);
    }

    #[test]
    fn skill_source_for_provider_unknown() {
        let sources = SkillSource::for_provider("unknown");
        assert_eq!(sources, vec![SkillSource::Stencila]);
    }

    // -- Helper to create a skill directory under a given dot-dir --

    fn create_skill(base: &Path, dot_dir: &str, name: &str, description: &str) {
        let skill_dir = base.join(dot_dir).join("skills").join(name);
        std::fs::create_dir_all(&skill_dir).expect("create skill dir");
        let content = format!(
            "---\nname: {name}\ndescription: {description}\n---\n\nInstructions for {name}.\n"
        );
        std::fs::write(skill_dir.join("SKILL.md"), content).expect("write SKILL.md");
    }

    // -- discover() tests --

    #[tokio::test]
    async fn discover_dedup_last_wins() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_skill(tmp.path(), ".stencila", "shared", "from stencila");
        create_skill(tmp.path(), ".claude", "shared", "from claude");

        let skills = discover(tmp.path(), &[SkillSource::Stencila, SkillSource::Claude]).await;

        let shared = skills.iter().find(|s| s.name == "shared").expect("found");
        assert_eq!(
            shared.inner.description, "from claude",
            "claude source should override stencila"
        );
        assert_eq!(shared.source(), SkillSource::Claude);
    }

    #[tokio::test]
    async fn discover_unique_skills_from_both_sources() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_skill(tmp.path(), ".stencila", "alpha", "stencila skill");
        create_skill(tmp.path(), ".claude", "beta", "claude skill");

        let skills = discover(tmp.path(), &[SkillSource::Stencila, SkillSource::Claude]).await;

        let alpha = skills.iter().find(|s| s.name == "alpha").expect("alpha");
        assert_eq!(alpha.source(), SkillSource::Stencila);
        let beta = skills.iter().find(|s| s.name == "beta").expect("beta");
        assert_eq!(beta.source(), SkillSource::Claude);
    }

    #[tokio::test]
    async fn discover_includes_builtins_when_no_local_sources_exist() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let skills = discover(tmp.path(), &SkillSource::all()).await;
        // Should include builtin skills even when no workspace sources exist
        assert!(!skills.is_empty(), "should include builtin skills");
    }

    // -- get_by_name() tests --

    #[tokio::test]
    async fn get_by_name_finds_skill_across_sources() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_skill(tmp.path(), ".claude", "my-skill", "a claude skill");

        let skill = get_by_name(
            tmp.path(),
            "my-skill",
            &[SkillSource::Stencila, SkillSource::Claude],
        )
        .await;

        assert!(skill.is_ok());
        assert_eq!(skill.expect("ok").name, "my-skill");
    }

    #[tokio::test]
    async fn get_by_name_last_wins() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_skill(tmp.path(), ".stencila", "shared", "from stencila");
        create_skill(tmp.path(), ".codex", "shared", "from codex");

        let skill = get_by_name(
            tmp.path(),
            "shared",
            &[SkillSource::Stencila, SkillSource::Codex],
        )
        .await
        .expect("found");

        assert_eq!(
            skill.inner.description, "from codex",
            "codex source should override stencila"
        );
        assert_eq!(skill.source(), SkillSource::Codex);
    }

    #[tokio::test]
    async fn get_by_name_returns_error_when_not_found() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = get_by_name(tmp.path(), "nonexistent", &SkillSource::all()).await;
        assert!(result.is_err());
    }
}
