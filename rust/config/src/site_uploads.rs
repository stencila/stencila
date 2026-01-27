//! Configuration for site uploads feature
//!
//! Site uploads allow users to upload files (e.g., CSV data updates) to a repository
//! via GitHub PRs. The feature requires a workspace.id to be configured for cloud
//! enforcement of public/anon settings.

use eyre::{Result, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Uploads configuration (detailed form), e.g.
/// ```toml
/// # Enable uploads on specific pages
/// [site.uploads]
/// enabled = true
/// include = ["data/**"]
/// ```
///
/// Note: Most upload settings (allowed extensions, max file size, target path, etc.)
/// are controlled by the server via the auth status response, not in the config.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SiteUploadsConfig {
    /// Whether uploads are enabled
    ///
    /// When false, the upload widget is not rendered.
    #[serde(default)]
    pub enabled: bool,

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

    /// File extensions to include in the `_files` index
    ///
    /// When specified, only files with these extensions are indexed.
    /// When `None` (default), all files are indexed.
    /// Extensions are matched case-insensitively, without leading dot.
    /// Example: `["csv", "json", "xlsx"]`
    pub extensions: Option<Vec<String>>,
}

impl SiteUploadsConfig {
    /// Validate the uploads configuration
    pub fn validate(&self) -> Result<()> {
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
pub enum SiteUploadsSpec {
    /// Simple boolean: uploads = true/false
    Enabled(bool),
    /// Detailed config: [site.uploads] with options
    Config(SiteUploadsConfig),
}

impl SiteUploadsSpec {
    /// Check if uploads are enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            SiteUploadsSpec::Enabled(enabled) => *enabled,
            SiteUploadsSpec::Config(config) => config.enabled,
        }
    }

    /// Convert to a full UploadsConfig, applying defaults for simple boolean form
    pub fn to_config(&self) -> SiteUploadsConfig {
        match self {
            SiteUploadsSpec::Enabled(enabled) => SiteUploadsConfig {
                enabled: *enabled,
                ..Default::default()
            },
            SiteUploadsSpec::Config(config) => config.clone(),
        }
    }

    /// Validate the uploads specification
    pub fn validate(&self) -> Result<()> {
        match self {
            SiteUploadsSpec::Enabled(_) => Ok(()),
            SiteUploadsSpec::Config(config) => config.validate(),
        }
    }
}

impl Default for SiteUploadsSpec {
    fn default() -> Self {
        SiteUploadsSpec::Enabled(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uploads_spec_simple_true() -> Result<(), serde_json::Error> {
        let spec: SiteUploadsSpec = serde_json::from_str("true")?;
        assert!(spec.is_enabled());
        let config = spec.to_config();
        assert!(config.enabled);
        Ok(())
    }

    #[test]
    fn test_uploads_spec_simple_false() -> Result<(), serde_json::Error> {
        let spec: SiteUploadsSpec = serde_json::from_str("false")?;
        assert!(!spec.is_enabled());
        Ok(())
    }

    #[test]
    fn test_uploads_spec_detailed() -> Result<(), serde_json::Error> {
        let json = r#"{
            "enabled": true,
            "include": ["data/**"]
        }"#;
        let spec: SiteUploadsSpec = serde_json::from_str(json)?;
        assert!(spec.is_enabled());

        let config = spec.to_config();
        assert_eq!(config.include, Some(vec!["data/**".to_string()]));
        Ok(())
    }

    #[test]
    fn test_uploads_spec_with_extensions() -> Result<(), serde_json::Error> {
        let json = r#"{
            "enabled": true,
            "extensions": ["csv", "json", "xlsx"]
        }"#;
        let spec: SiteUploadsSpec = serde_json::from_str(json)?;
        assert!(spec.is_enabled());

        let config = spec.to_config();
        assert_eq!(
            config.extensions,
            Some(vec![
                "csv".to_string(),
                "json".to_string(),
                "xlsx".to_string()
            ])
        );
        Ok(())
    }

    #[test]
    fn test_validate_defaults() {
        let config = SiteUploadsConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_glob_patterns() {
        let config = SiteUploadsConfig {
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
        let config = SiteUploadsConfig {
            include: Some(vec!["data/[invalid".to_string()]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("include pattern"));
    }

    #[test]
    fn test_validate_multiple_double_star_segments() {
        let config = SiteUploadsConfig {
            include: Some(vec!["**/data/**".to_string()]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("multiple ** segments"));
    }
}
