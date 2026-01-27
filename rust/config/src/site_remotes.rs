//! Configuration for site remotes action
//!
//! Site remotes allow users to add Google Docs or Microsoft 365 documents
//! to a repository via GitHub PRs, with optional bi-directional sync.
//! The feature requires a workspace.id to be configured for cloud enforcement
//! of public/anon settings.

use eyre::{Result, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Sync direction options for remote documents
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SiteRemoteSyncDirection {
    /// Changes in remote doc create PRs to update repo
    FromRemote,
    /// Changes sync both ways (default)
    #[default]
    Bi,
    /// Changes in repo update remote doc
    ToRemote,
}

/// Output format options for pulled documents
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SiteRemoteFormat {
    /// Stencila Markdown (default)
    #[default]
    Smd,
    /// Standard Markdown
    Md,
    /// HTML
    Html,
}

impl SiteRemoteFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            SiteRemoteFormat::Smd => ".smd",
            SiteRemoteFormat::Md => ".md",
            SiteRemoteFormat::Html => ".html",
        }
    }
}

/// Remotes action configuration (detailed form), e.g.
/// ```toml
/// # Enable adding remote documents
/// [site.remotes]
/// enabled = true
/// path = "content"
/// default-format = "smd"
/// allowed-formats = ["smd", "md"]
/// public = true
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SiteRemotesConfig {
    /// Whether remote document adding is enabled
    ///
    /// When false, the remote widget is not rendered.
    #[serde(default)]
    pub enabled: bool,

    /// Whether public (non-team members) can add remote documents
    ///
    /// This is enforced server-side by Stencila Cloud. When false,
    /// the remote widget is hidden from non-authenticated users.
    /// Default: false (more restrictive)
    pub public: Option<bool>,

    /// Whether anonymous (no GitHub auth) submissions are allowed
    ///
    /// This is enforced server-side by Stencila Cloud. When false,
    /// users must connect their GitHub account to add remote documents.
    /// Default: false
    pub anon: Option<bool>,

    /// Default target directory for new remote documents
    ///
    /// Path is relative to repo root.
    /// Example: "content" or "docs"
    pub path: Option<String>,

    /// Allow users to specify custom target paths
    ///
    /// When true, users can edit the target path for each document.
    /// Default: false
    pub user_path: Option<bool>,

    /// Default output format for pulled documents
    ///
    /// Default: smd (Stencila Markdown)
    pub default_format: Option<SiteRemoteFormat>,

    /// Allowed output formats
    ///
    /// If specified, users can only choose from these formats.
    /// Default: all formats allowed
    pub allowed_formats: Option<Vec<SiteRemoteFormat>>,

    /// Default sync direction
    ///
    /// Default: bi (bi-directional)
    pub default_sync_direction: Option<SiteRemoteSyncDirection>,

    /// Require a description/commit message
    ///
    /// When true, users must provide a message describing their addition.
    /// This becomes the PR description.
    /// Default: false
    pub require_message: Option<bool>,

    /// Glob patterns for paths to show widget on
    ///
    /// If specified, widget is only shown on pages matching these patterns.
    /// Example: `["docs/**", "content/**"]`
    pub include: Option<Vec<String>>,

    /// Glob patterns for paths to hide widget from
    ///
    /// Widget is hidden on pages matching these patterns.
    /// Example: `["api/**", "internal/**"]`
    pub exclude: Option<Vec<String>>,
}

impl SiteRemotesConfig {
    /// Get the effective public setting (defaults to false)
    pub fn is_public(&self) -> bool {
        self.public.unwrap_or(false)
    }

    /// Get the effective anon setting (defaults to false)
    pub fn is_anon(&self) -> bool {
        self.anon.unwrap_or(false)
    }

    /// Get the effective target path (defaults to empty string / repo root)
    pub fn target_path(&self) -> String {
        self.path.clone().unwrap_or_default()
    }

    /// Get the effective user_path setting (defaults to false)
    pub fn user_path_enabled(&self) -> bool {
        self.user_path.unwrap_or(false)
    }

    /// Get the effective default format (defaults to Smd)
    pub fn default_format(&self) -> SiteRemoteFormat {
        self.default_format.unwrap_or_default()
    }

    /// Get the effective default sync direction (defaults to Bi)
    pub fn default_sync_direction(&self) -> SiteRemoteSyncDirection {
        self.default_sync_direction.unwrap_or_default()
    }

    /// Get the effective require_message setting (defaults to false)
    pub fn require_message(&self) -> bool {
        self.require_message.unwrap_or(false)
    }

    /// Check if a format is allowed
    pub fn is_format_allowed(&self, format: SiteRemoteFormat) -> bool {
        match &self.allowed_formats {
            Some(formats) => formats.contains(&format),
            None => true, // All formats allowed by default
        }
    }

    /// Validate the remotes configuration
    pub fn validate(&self) -> Result<()> {
        // Validate allowed_formats is not an empty array
        if let Some(formats) = &self.allowed_formats
            && formats.is_empty()
        {
            bail!(
                "Invalid remotes config: allowed-formats cannot be empty (omit the field to allow all formats)"
            );
        }

        // Validate default_format is in allowed_formats (if both specified)
        if let (Some(default), Some(allowed)) = (&self.default_format, &self.allowed_formats)
            && !allowed.contains(default)
        {
            bail!(
                "Invalid remotes config: default-format \"{default:?}\" is not in allowed-formats {:?}",
                allowed
            );
        }

        // Validate include patterns are valid globs
        if let Some(patterns) = &self.include {
            for pattern in patterns {
                validate_glob_pattern(pattern, "include")?;
            }
        }

        // Validate exclude patterns are valid globs
        if let Some(patterns) = &self.exclude {
            for pattern in patterns {
                validate_glob_pattern(pattern, "exclude")?;
            }
        }

        Ok(())
    }
}

/// Validate a glob pattern string
fn validate_glob_pattern(pattern: &str, field: &str) -> Result<()> {
    let pattern = pattern.trim_matches('/');

    // Skip full validation for ** patterns (handled specially at runtime)
    if pattern.contains("**") {
        // Reject patterns with multiple ** segments (not supported at runtime)
        if pattern.matches("**").count() > 1 {
            bail!(
                "Invalid remotes config: {field} pattern \"{pattern}\" contains multiple ** segments which is not supported"
            );
        }

        // Reject patterns with glob metacharacters in the suffix (after **)
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let suffix = parts[1].trim_start_matches('/');
            if suffix.contains('*') || suffix.contains('?') || suffix.contains('[') {
                bail!(
                    "Invalid remotes config: {field} pattern \"{pattern}\" has glob wildcards after **. \
                    Use \"docs/**\" to match all files, not \"docs/**/*.md\""
                );
            }
        }

        return Ok(());
    }

    // Validate patterns containing glob metacharacters using glob crate
    if (pattern.contains('*') || pattern.contains('[') || pattern.contains('?'))
        && let Err(e) = glob::Pattern::new(pattern)
    {
        bail!("Invalid remotes config: {field} pattern \"{pattern}\" is not a valid glob: {e}");
    }

    Ok(())
}

/// Remotes specification - handles both boolean and detailed forms
///
/// Allows configuration as either:
/// - Simple boolean: `remotes = true` or `remotes = false`
/// - Detailed config: `[site.remotes]` with options
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum SiteRemotesSpec {
    /// Simple boolean: remotes = true/false
    Enabled(bool),
    /// Detailed config: [site.remotes] with options
    Config(SiteRemotesConfig),
}

impl SiteRemotesSpec {
    /// Check if remotes are enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            SiteRemotesSpec::Enabled(enabled) => *enabled,
            SiteRemotesSpec::Config(config) => config.enabled,
        }
    }

    /// Convert to a full RemotesConfig, applying defaults for simple boolean form
    pub fn to_config(&self) -> SiteRemotesConfig {
        match self {
            SiteRemotesSpec::Enabled(enabled) => SiteRemotesConfig {
                enabled: *enabled,
                ..Default::default()
            },
            SiteRemotesSpec::Config(config) => config.clone(),
        }
    }

    /// Validate the remotes specification
    pub fn validate(&self) -> Result<()> {
        match self {
            SiteRemotesSpec::Enabled(_) => Ok(()),
            SiteRemotesSpec::Config(config) => config.validate(),
        }
    }
}

impl Default for SiteRemotesSpec {
    fn default() -> Self {
        SiteRemotesSpec::Enabled(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remotes_spec_simple_true() -> Result<(), serde_json::Error> {
        let spec: SiteRemotesSpec = serde_json::from_str("true")?;
        assert!(spec.is_enabled());
        let config = spec.to_config();
        assert!(config.enabled);
        assert!(!config.is_public());
        assert!(!config.is_anon());
        Ok(())
    }

    #[test]
    fn test_remotes_spec_simple_false() -> Result<(), serde_json::Error> {
        let spec: SiteRemotesSpec = serde_json::from_str("false")?;
        assert!(!spec.is_enabled());
        Ok(())
    }

    #[test]
    fn test_remotes_spec_detailed() -> Result<(), serde_json::Error> {
        let json = r#"{
            "enabled": true,
            "public": true,
            "anon": false,
            "path": "content",
            "default-format": "md",
            "allowed-formats": ["smd", "md"],
            "default-sync-direction": "from-remote",
            "user-path": true,
            "require-message": true
        }"#;
        let spec: SiteRemotesSpec = serde_json::from_str(json)?;
        assert!(spec.is_enabled());

        let config = spec.to_config();
        assert!(config.is_public());
        assert!(!config.is_anon());
        assert_eq!(config.target_path(), "content");
        assert_eq!(config.default_format(), SiteRemoteFormat::Md);
        assert!(config.is_format_allowed(SiteRemoteFormat::Smd));
        assert!(config.is_format_allowed(SiteRemoteFormat::Md));
        assert!(!config.is_format_allowed(SiteRemoteFormat::Html));
        assert_eq!(
            config.default_sync_direction(),
            SiteRemoteSyncDirection::FromRemote
        );
        assert!(config.user_path_enabled());
        assert!(config.require_message());
        Ok(())
    }

    #[test]
    fn test_sync_direction_serialization() -> Result<(), serde_json::Error> {
        assert_eq!(
            serde_json::to_string(&SiteRemoteSyncDirection::FromRemote)?,
            "\"from-remote\""
        );
        assert_eq!(
            serde_json::to_string(&SiteRemoteSyncDirection::Bi)?,
            "\"bi\""
        );
        assert_eq!(
            serde_json::to_string(&SiteRemoteSyncDirection::ToRemote)?,
            "\"to-remote\""
        );
        Ok(())
    }

    #[test]
    fn test_remote_format_serialization() -> Result<(), serde_json::Error> {
        assert_eq!(serde_json::to_string(&SiteRemoteFormat::Smd)?, "\"smd\"");
        assert_eq!(serde_json::to_string(&SiteRemoteFormat::Md)?, "\"md\"");
        assert_eq!(serde_json::to_string(&SiteRemoteFormat::Html)?, "\"html\"");
        Ok(())
    }

    #[test]
    fn test_remote_format_extension() {
        assert_eq!(SiteRemoteFormat::Smd.extension(), ".smd");
        assert_eq!(SiteRemoteFormat::Md.extension(), ".md");
        assert_eq!(SiteRemoteFormat::Html.extension(), ".html");
    }

    #[test]
    fn test_validate_empty_allowed_formats() {
        let config = SiteRemotesConfig {
            allowed_formats: Some(vec![]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("allowed-formats cannot be empty"));
    }

    #[test]
    fn test_validate_valid_config() {
        let config = SiteRemotesConfig {
            path: Some("content".to_string()),
            allowed_formats: Some(vec![SiteRemoteFormat::Smd, SiteRemoteFormat::Md]),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_defaults() {
        let config = SiteRemotesConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_glob_patterns() {
        let config = SiteRemotesConfig {
            include: Some(vec!["content/**".to_string(), "docs/*.md".to_string()]),
            exclude: Some(vec!["api/**".to_string()]),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_glob_pattern() {
        let config = SiteRemotesConfig {
            include: Some(vec!["content/[invalid".to_string()]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("include pattern"));
    }

    #[test]
    fn test_validate_default_format_not_in_allowed() {
        let config = SiteRemotesConfig {
            default_format: Some(SiteRemoteFormat::Html),
            allowed_formats: Some(vec![SiteRemoteFormat::Smd, SiteRemoteFormat::Md]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("default-format"));
        assert!(err.to_string().contains("allowed-formats"));
    }

    #[test]
    fn test_validate_default_format_in_allowed() {
        let config = SiteRemotesConfig {
            default_format: Some(SiteRemoteFormat::Md),
            allowed_formats: Some(vec![SiteRemoteFormat::Smd, SiteRemoteFormat::Md]),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_default_format_without_allowed() {
        // When allowed_formats is not specified, any default_format is valid
        let config = SiteRemotesConfig {
            default_format: Some(SiteRemoteFormat::Html),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
