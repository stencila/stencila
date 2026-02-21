use std::collections::HashMap;
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

/// Subdirectory name within each dot-directory that holds skills.
const SKILLS_SUBDIR: &str = "skills";

/// Where a skill was discovered from.
///
/// Each variant corresponds to a dot-directory that may contain a `skills/`
/// subdirectory. Sources listed later in [`SkillSource::all`] take precedence
/// when names conflict (last wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum SkillSource {
    /// `.stencila/skills/` — base layer, always loaded first
    Stencila,
    /// `.claude/skills/` — Anthropic provider
    Claude,
    /// `.codex/skills/` — OpenAI provider
    Codex,
    /// `.gemini/skills/` — Google Gemini provider
    Gemini,
}

impl std::fmt::Display for SkillSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stencila => f.write_str("stencila"),
            Self::Claude => f.write_str("claude"),
            Self::Codex => f.write_str("codex"),
            Self::Gemini => f.write_str("gemini"),
        }
    }
}

impl SkillSource {
    /// The dot-directory name for this source (e.g. `.stencila`, `.claude`).
    pub fn dir_name(&self) -> &'static str {
        match self {
            Self::Stencila => ".stencila",
            Self::Claude => ".claude",
            Self::Codex => ".codex",
            Self::Gemini => ".gemini",
        }
    }

    /// All sources in precedence order (Stencila first, provider-specific after).
    pub fn all() -> Vec<Self> {
        vec![Self::Stencila, Self::Claude, Self::Codex, Self::Gemini]
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

    /// Which source this skill was loaded from
    #[serde(skip)]
    source: Option<SkillSource>,
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
        let mut state = serializer.serialize_struct("SkillInstance", 6)?;

        state.serialize_field("name", &self.inner.name)?;
        state.serialize_field("description", &self.inner.description)?;
        state.serialize_field("licenses", &self.inner.options.licenses)?;
        state.serialize_field("compatibility", &self.inner.compatibility)?;
        state.serialize_field("source", &self.source.map(|s| s.to_string()))?;
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

        Ok(Self {
            inner,
            path,
            home,
            source: None,
        })
    }

    /// Return a copy with the source set.
    fn with_source(mut self, source: SkillSource) -> Self {
        self.source = Some(source);
        self
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
    pub fn source(&self) -> Option<SkillSource> {
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
/// Walks from `cwd` upward for each source, collecting skills. When the same
/// skill name appears in multiple sources, the later source wins.
pub async fn discover(cwd: &Path, sources: &[SkillSource]) -> Vec<SkillInstance> {
    let mut by_name: HashMap<String, SkillInstance> = HashMap::new();
    for &source in sources {
        if let Some(dot_dir) = stencila_dirs::closest_dot_dir(cwd, source.dir_name()) {
            let skills_dir = dot_dir.join(SKILLS_SUBDIR);
            for skill in list(&skills_dir).await {
                by_name.insert(skill.name.clone(), skill.with_source(source));
            }
        }
    }
    let mut skills: Vec<SkillInstance> = by_name.into_values().collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Find a skill by name across multiple sources (last wins).
pub async fn get_from(cwd: &Path, name: &str, sources: &[SkillSource]) -> Result<SkillInstance> {
    let mut found: Option<SkillInstance> = None;
    for &source in sources {
        if let Some(dot_dir) = stencila_dirs::closest_dot_dir(cwd, source.dir_name()) {
            let skills_dir = dot_dir.join(SKILLS_SUBDIR);
            if let Ok(skill) = get(&skills_dir, name).await {
                found = Some(skill.with_source(source));
            }
        }
    }
    found.ok_or_else(|| eyre::eyre!("Unable to find skill with name `{name}`"))
}

/// List all skills in the workspace closest to the current directory
///
/// Discovers skills from all sources (Stencila + all providers).
pub async fn list_current() -> Vec<SkillInstance> {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            tracing::error!("Unable to get current directory: {error}");
            return Vec::new();
        }
    };

    discover(&cwd, &SkillSource::all()).await
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

#[cfg(test)]
mod tests {
    use super::*;

    // -- SkillSource unit tests --

    #[test]
    fn skill_source_all_returns_four_variants() {
        let all = SkillSource::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], SkillSource::Stencila);
        assert_eq!(all[1], SkillSource::Claude);
        assert_eq!(all[2], SkillSource::Codex);
        assert_eq!(all[3], SkillSource::Gemini);
    }

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

    #[test]
    fn skill_source_dir_names() {
        assert_eq!(SkillSource::Stencila.dir_name(), ".stencila");
        assert_eq!(SkillSource::Claude.dir_name(), ".claude");
        assert_eq!(SkillSource::Codex.dir_name(), ".codex");
        assert_eq!(SkillSource::Gemini.dir_name(), ".gemini");
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

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "shared");
        assert_eq!(
            skills[0].inner.description, "from claude",
            "claude source should override stencila"
        );
        assert_eq!(skills[0].source(), Some(SkillSource::Claude));
    }

    #[tokio::test]
    async fn discover_unique_skills_from_both_sources() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_skill(tmp.path(), ".stencila", "alpha", "stencila skill");
        create_skill(tmp.path(), ".claude", "beta", "claude skill");

        let skills = discover(tmp.path(), &[SkillSource::Stencila, SkillSource::Claude]).await;

        assert_eq!(skills.len(), 2);
        // Sorted alphabetically
        assert_eq!(skills[0].name, "alpha");
        assert_eq!(skills[0].source(), Some(SkillSource::Stencila));
        assert_eq!(skills[1].name, "beta");
        assert_eq!(skills[1].source(), Some(SkillSource::Claude));
    }

    #[tokio::test]
    async fn discover_empty_when_no_sources_exist() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let skills = discover(tmp.path(), &SkillSource::all()).await;
        assert!(skills.is_empty());
    }

    // -- get_from() tests --

    #[tokio::test]
    async fn get_from_finds_skill_across_sources() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_skill(tmp.path(), ".claude", "my-skill", "a claude skill");

        let skill = get_from(
            tmp.path(),
            "my-skill",
            &[SkillSource::Stencila, SkillSource::Claude],
        )
        .await;

        assert!(skill.is_ok());
        assert_eq!(skill.expect("ok").name, "my-skill");
    }

    #[tokio::test]
    async fn get_from_last_wins() {
        let tmp = tempfile::tempdir().expect("tempdir");
        create_skill(tmp.path(), ".stencila", "shared", "from stencila");
        create_skill(tmp.path(), ".codex", "shared", "from codex");

        let skill = get_from(
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
        assert_eq!(skill.source(), Some(SkillSource::Codex));
    }

    #[tokio::test]
    async fn get_from_returns_error_when_not_found() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = get_from(tmp.path(), "nonexistent", &SkillSource::all()).await;
        assert!(result.is_err());
    }
}
