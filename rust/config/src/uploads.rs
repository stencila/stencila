//! Configuration for site uploads feature
//!
//! Site uploads allow users to upload files (e.g., CSV data updates) to a repository
//! via GitHub PRs. The feature requires a workspace.id to be configured for cloud
//! enforcement of public/anon settings.

use eyre::{Result, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Default maximum file size in bytes (10MB)
pub const DEFAULT_MAX_SIZE: u64 = 10 * 1024 * 1024;

/// Uploads configuration (detailed form), e.g.
/// ```toml
/// # Enable uploads for data files
/// [site.uploads]
/// enabled = true
/// path = "data"
/// allowed-types = ["csv", "json", "xlsx"]
/// max-size = 10485760
/// public = false
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct UploadsConfig {
    /// Whether uploads are enabled
    ///
    /// When false, the upload widget is not rendered.
    #[serde(default)]
    pub enabled: bool,

    /// Whether public (non-team members) can see the upload widget
    ///
    /// This is enforced server-side by Stencila Cloud. When false,
    /// the upload widget is hidden from non-authenticated users.
    /// Default: false (more restrictive than reviews)
    pub public: Option<bool>,

    /// Whether anonymous (no GitHub auth) submissions are allowed
    ///
    /// This is enforced server-side by Stencila Cloud. When false,
    /// users must connect their GitHub account to submit uploads.
    /// Default: false
    pub anon: Option<bool>,

    /// Allowed file extensions for uploads
    ///
    /// Files with extensions not in this list will be rejected.
    /// Extensions should be lowercase without leading dot.
    /// Example: `["csv", "json", "xlsx", "md"]`
    pub allowed_types: Option<Vec<String>>,

    /// Maximum file size in bytes
    ///
    /// Files larger than this will be rejected.
    /// Default: 10MB (10485760 bytes)
    pub max_size: Option<u64>,

    /// Unified path for visibility and destination
    ///
    /// Controls both which pages show the upload widget (widget on `/{path}/**` pages)
    /// and where uploaded files are saved (in `{path}/` directory).
    ///
    /// Can be overridden separately using `include`/`exclude` for visibility
    /// and `target-path` for destination.
    pub path: Option<String>,

    /// Override: explicit target directory for uploads
    ///
    /// If set, uploaded files go here instead of the `path` directory.
    /// Path is relative to repo root.
    pub target_path: Option<String>,

    /// Override: glob patterns for pages to show widget on
    ///
    /// If specified, overrides the visibility derived from `path`.
    /// Example: `["admin/**", "dashboard/**"]`
    pub include: Option<Vec<String>>,

    /// Glob patterns for pages to hide widget from
    ///
    /// Widget is hidden on pages matching these patterns.
    /// Example: `["api/**", "internal/**"]`
    pub exclude: Option<Vec<String>>,

    /// Allow users to specify custom upload paths
    ///
    /// When true, users can edit the target path for each upload
    /// (within the configured target directory).
    /// Default: false
    pub user_path: Option<bool>,

    /// Allow overwriting/replacing existing files
    ///
    /// When true, users can upload files that replace existing ones.
    /// When false, uploads to existing paths are rejected.
    /// Default: true
    pub allow_overwrite: Option<bool>,

    /// Require a description/commit message
    ///
    /// When true, users must provide a message describing their upload.
    /// This becomes the PR description.
    /// Default: false
    pub require_message: Option<bool>,
}

impl UploadsConfig {
    /// Get the effective public setting (defaults to false)
    pub fn is_public(&self) -> bool {
        self.public.unwrap_or(false)
    }

    /// Get the effective anon setting (defaults to false)
    pub fn is_anon(&self) -> bool {
        self.anon.unwrap_or(false)
    }

    /// Get the effective max_size (defaults to 10MB)
    pub fn max_size(&self) -> u64 {
        self.max_size.unwrap_or(DEFAULT_MAX_SIZE)
    }

    /// Get the effective target path
    ///
    /// Returns `target_path` if set, otherwise `path`, otherwise empty string (repo root)
    pub fn target_path(&self) -> String {
        self.target_path
            .clone()
            .or_else(|| self.path.clone())
            .unwrap_or_default()
    }

    /// Get the effective include patterns
    ///
    /// Returns explicit `include` if set, otherwise derives from `path`
    pub fn include_patterns(&self) -> Option<Vec<String>> {
        if self.include.is_some() {
            return self.include.clone();
        }
        // Derive from path if set
        self.path.as_ref().map(|p| {
            let p = p.trim_matches('/');
            vec![format!("{p}/**")]
        })
    }

    /// Get the effective user_path setting (defaults to false)
    pub fn user_path_enabled(&self) -> bool {
        self.user_path.unwrap_or(false)
    }

    /// Get the effective allow_overwrite setting (defaults to true)
    pub fn allow_overwrite(&self) -> bool {
        self.allow_overwrite.unwrap_or(true)
    }

    /// Get the effective require_message setting (defaults to false)
    pub fn require_message(&self) -> bool {
        self.require_message.unwrap_or(false)
    }

    /// Check if a file extension is allowed
    pub fn is_extension_allowed(&self, extension: &str) -> bool {
        match &self.allowed_types {
            Some(types) => types.iter().any(|t| t.eq_ignore_ascii_case(extension)),
            None => true, // All types allowed by default
        }
    }

    /// Validate the uploads configuration
    pub fn validate(&self) -> Result<()> {
        // Validate max_size is reasonable (max 100MB)
        let max = self.max_size();
        if max > 100 * 1024 * 1024 {
            bail!(
                "Invalid uploads config: max-size ({} bytes) exceeds maximum allowed (100MB)",
                max
            );
        }

        // Validate allowed_types is not an empty array
        if let Some(types) = &self.allowed_types
            && types.is_empty()
        {
            bail!(
                "Invalid uploads config: allowed-types cannot be empty (omit the field to allow all types)"
            );
        }

        // Validate allowed_types don't have leading dots
        if let Some(types) = &self.allowed_types {
            for t in types {
                if t.starts_with('.') {
                    bail!(
                        "Invalid uploads config: allowed-types should not include leading dot: '{t}'"
                    );
                }
            }
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
///
/// Checks that the pattern is syntactically valid. Supports wildcards (`*`, `**`)
/// and character classes (`[abc]`).
fn validate_glob_pattern(pattern: &str, field: &str) -> Result<()> {
    let pattern = pattern.trim_matches('/');

    // Skip full validation for ** patterns (handled specially at runtime)
    if pattern.contains("**") {
        // Reject patterns with multiple ** segments (not supported at runtime)
        if pattern.matches("**").count() > 1 {
            bail!(
                "Invalid uploads config: {field} pattern \"{pattern}\" contains multiple ** segments which is not supported"
            );
        }

        // Reject patterns with glob metacharacters in the suffix (after **)
        // Runtime uses literal string matching for suffix, so patterns like
        // "docs/**/*.md" won't work as expected
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let suffix = parts[1].trim_start_matches('/');
            if suffix.contains('*') || suffix.contains('?') || suffix.contains('[') {
                bail!(
                    "Invalid uploads config: {field} pattern \"{pattern}\" has glob wildcards after **. \
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
        bail!("Invalid uploads config: {field} pattern \"{pattern}\" is not a valid glob: {e}");
    }

    Ok(())
}

/// Uploads specification - handles both boolean and detailed forms
///
/// Allows configuration as either:
/// - Simple boolean: `uploads = true` or `uploads = false`
/// - Detailed config: `[site.uploads]` with options
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum UploadsSpec {
    /// Simple boolean: uploads = true/false
    Enabled(bool),
    /// Detailed config: [site.uploads] with options
    Config(UploadsConfig),
}

impl UploadsSpec {
    /// Check if uploads are enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            UploadsSpec::Enabled(enabled) => *enabled,
            UploadsSpec::Config(config) => config.enabled,
        }
    }

    /// Convert to a full UploadsConfig, applying defaults for simple boolean form
    pub fn to_config(&self) -> UploadsConfig {
        match self {
            UploadsSpec::Enabled(enabled) => UploadsConfig {
                enabled: *enabled,
                ..Default::default()
            },
            UploadsSpec::Config(config) => config.clone(),
        }
    }

    /// Validate the uploads specification
    pub fn validate(&self) -> Result<()> {
        match self {
            UploadsSpec::Enabled(_) => Ok(()),
            UploadsSpec::Config(config) => config.validate(),
        }
    }
}

impl Default for UploadsSpec {
    fn default() -> Self {
        UploadsSpec::Enabled(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uploads_spec_simple_true() -> Result<(), serde_json::Error> {
        let spec: UploadsSpec = serde_json::from_str("true")?;
        assert!(spec.is_enabled());
        let config = spec.to_config();
        assert!(config.enabled);
        assert!(!config.is_public()); // Default false for uploads
        assert!(!config.is_anon());
        Ok(())
    }

    #[test]
    fn test_uploads_spec_simple_false() -> Result<(), serde_json::Error> {
        let spec: UploadsSpec = serde_json::from_str("false")?;
        assert!(!spec.is_enabled());
        Ok(())
    }

    #[test]
    fn test_uploads_spec_detailed() -> Result<(), serde_json::Error> {
        let json = r#"{
            "enabled": true,
            "public": true,
            "anon": false,
            "allowed-types": ["csv", "json"],
            "max-size": 5242880,
            "path": "data",
            "user-path": true,
            "require-message": true
        }"#;
        let spec: UploadsSpec = serde_json::from_str(json)?;
        assert!(spec.is_enabled());

        let config = spec.to_config();
        assert!(config.is_public());
        assert!(!config.is_anon());
        assert!(config.is_extension_allowed("csv"));
        assert!(config.is_extension_allowed("JSON")); // Case insensitive
        assert!(!config.is_extension_allowed("exe"));
        assert_eq!(config.max_size(), 5242880);
        assert_eq!(config.target_path(), "data");
        assert!(config.user_path_enabled());
        assert!(config.require_message());
        Ok(())
    }

    #[test]
    fn test_validate_empty_allowed_types() {
        let config = UploadsConfig {
            allowed_types: Some(vec![]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("allowed-types cannot be empty"));
    }

    #[test]
    fn test_validate_allowed_types_with_dots() {
        let config = UploadsConfig {
            allowed_types: Some(vec![".csv".to_string()]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("should not include leading dot"));
    }

    #[test]
    fn test_validate_max_size_too_large() {
        let config = UploadsConfig {
            max_size: Some(200 * 1024 * 1024), // 200MB
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("exceeds maximum allowed"));
    }

    #[test]
    fn test_validate_valid_config() {
        let config = UploadsConfig {
            allowed_types: Some(vec!["csv".to_string(), "json".to_string()]),
            max_size: Some(5 * 1024 * 1024),
            path: Some("data".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_defaults() {
        let config = UploadsConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_target_path_resolution() {
        // No path set - returns empty string
        let config = UploadsConfig::default();
        assert_eq!(config.target_path(), "");

        // Only path set
        let config = UploadsConfig {
            path: Some("data".to_string()),
            ..Default::default()
        };
        assert_eq!(config.target_path(), "data");

        // target_path overrides path
        let config = UploadsConfig {
            path: Some("data".to_string()),
            target_path: Some("uploads".to_string()),
            ..Default::default()
        };
        assert_eq!(config.target_path(), "uploads");
    }

    #[test]
    fn test_include_patterns_resolution() {
        // No path or include - returns None
        let config = UploadsConfig::default();
        assert!(config.include_patterns().is_none());

        // path set - derives include pattern
        let config = UploadsConfig {
            path: Some("data".to_string()),
            ..Default::default()
        };
        assert_eq!(config.include_patterns(), Some(vec!["data/**".to_string()]));

        // explicit include overrides path
        let config = UploadsConfig {
            path: Some("data".to_string()),
            include: Some(vec!["admin/**".to_string()]),
            ..Default::default()
        };
        assert_eq!(
            config.include_patterns(),
            Some(vec!["admin/**".to_string()])
        );
    }

    #[test]
    fn test_validate_valid_glob_patterns() {
        let config = UploadsConfig {
            include: Some(vec![
                "data/**".to_string(),
                "uploads/*.csv".to_string(),
                "admin/**".to_string(),
            ]),
            exclude: Some(vec!["internal/**".to_string(), "*.tmp".to_string()]),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_glob_pattern() {
        let config = UploadsConfig {
            include: Some(vec!["data/[invalid".to_string()]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("include pattern"));
    }

    #[test]
    fn test_validate_multiple_double_star_segments() {
        let config = UploadsConfig {
            include: Some(vec!["**/data/**".to_string()]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("multiple ** segments"));
    }
}
