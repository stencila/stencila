//! Configuration for site reviews feature
//!
//! Site reviews allow readers to submit comments and suggestions on site pages.
//! The feature requires a workspace.id to be configured for cloud enforcement
//! of public/anon settings.

use eyre::{Result, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Position for the reviews widget on the page
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewsPosition {
    /// Bottom-right corner (default)
    #[default]
    BottomRight,
    /// Bottom-left corner
    BottomLeft,
    /// Top-right corner
    TopRight,
    /// Top-left corner
    TopLeft,
}

/// Allowed review item types
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewType {
    /// Comment on selected text
    Comment,
    /// Suggestion to replace selected text
    Suggestion,
}

/// Reviews configuration (detailed form), e.g.
/// ```toml
/// # Enable reviews with selection limits and filters
/// [site.reviews]
/// enabled = true
/// public = true
/// anon = false
/// position = "bottom-right"
/// types = ["comment", "suggestion"]
/// min-selection = 3
/// max-selection = 5000
/// shortcuts = false
/// include = ["docs/**"]
/// exclude = ["api/**"]
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ReviewsConfig {
    /// Whether reviews are enabled
    ///
    /// When false, the review widget is not rendered.
    #[serde(default)]
    pub enabled: bool,

    /// Whether public (non-team members) can submit reviews
    ///
    /// This is enforced server-side by Stencila Cloud. When false,
    /// the review widget is hidden from non-authenticated users.
    /// Default: true
    pub public: Option<bool>,

    /// Whether anonymous (no GitHub auth) submissions are allowed
    ///
    /// This is enforced server-side by Stencila Cloud. When false,
    /// users must connect their GitHub account to submit reviews.
    /// Default: false
    pub anon: Option<bool>,

    /// Position of the reviews widget on the page
    ///
    /// Default: bottom-right
    pub position: Option<ReviewsPosition>,

    /// Allowed review item types
    ///
    /// Default: both comment and suggestion
    pub types: Option<Vec<ReviewType>>,

    /// Minimum characters required to trigger the widget
    ///
    /// Prevents accidental tiny selections from showing the review buttons.
    /// Default: 1
    pub min_selection: Option<u32>,

    /// Maximum characters allowed in a selection
    ///
    /// Prevents selecting excessively large amounts of text.
    /// Default: 5000
    pub max_selection: Option<u32>,

    /// Enable keyboard shortcuts for reviews
    ///
    /// When enabled:
    /// - Ctrl+Shift+C: Add comment on current selection
    /// - Ctrl+Shift+S: Add suggestion on current selection
    /// - Escape: Cancel current input / clear selection
    ///
    /// Default: true
    pub shortcuts: Option<bool>,

    /// Glob patterns for paths to show reviews on
    ///
    /// If specified, reviews are only shown on pages matching these patterns.
    /// Example: `["docs/**", "guides/**"]`
    pub include: Option<Vec<String>>,

    /// Glob patterns for paths to hide reviews from
    ///
    /// Reviews are hidden on pages matching these patterns.
    /// Example: `["api/**", "changelog/**"]`
    pub exclude: Option<Vec<String>>,
}

impl ReviewsConfig {
    /// Get the effective public setting (defaults to true)
    pub fn is_public(&self) -> bool {
        self.public.unwrap_or(true)
    }

    /// Get the effective anon setting (defaults to false)
    pub fn is_anon(&self) -> bool {
        self.anon.unwrap_or(false)
    }

    /// Get the effective position (defaults to BottomRight)
    pub fn position(&self) -> ReviewsPosition {
        self.position.unwrap_or_default()
    }

    /// Get the effective min_selection (defaults to 1)
    pub fn min_selection(&self) -> u32 {
        self.min_selection.unwrap_or(1)
    }

    /// Get the effective max_selection (defaults to 5000)
    pub fn max_selection(&self) -> u32 {
        self.max_selection.unwrap_or(5000)
    }

    /// Get the effective shortcuts setting (defaults to true)
    pub fn shortcuts_enabled(&self) -> bool {
        self.shortcuts.unwrap_or(true)
    }

    /// Check if comments are allowed
    pub fn allows_comments(&self) -> bool {
        match &self.types {
            Some(types) => types.contains(&ReviewType::Comment),
            None => true, // Both allowed by default
        }
    }

    /// Check if suggestions are allowed
    pub fn allows_suggestions(&self) -> bool {
        match &self.types {
            Some(types) => types.contains(&ReviewType::Suggestion),
            None => true, // Both allowed by default
        }
    }

    /// Validate the reviews configuration
    pub fn validate(&self) -> Result<()> {
        // Validate min_selection <= max_selection
        let min = self.min_selection();
        let max = self.max_selection();
        if min > max {
            bail!(
                "Invalid reviews config: min-selection ({min}) cannot be greater than max-selection ({max})"
            );
        }

        // Validate types is not an empty array
        if let Some(types) = &self.types
            && types.is_empty()
        {
            bail!(
                "Invalid reviews config: types cannot be empty (omit the field to allow both comment and suggestion)"
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
///
/// Checks that the pattern is syntactically valid. Supports wildcards (`*`, `**`)
/// and character classes (`[abc]`).
/// Note: `**` patterns are handled specially at runtime and don't use the glob crate.
fn validate_glob_pattern(pattern: &str, field: &str) -> Result<()> {
    let pattern = pattern.trim_matches('/');

    // Skip full validation for ** patterns (handled specially at runtime)
    // but still validate any other glob syntax in the pattern
    if pattern.contains("**") {
        // Reject patterns with multiple ** segments (not supported at runtime)
        if pattern.matches("**").count() > 1 {
            bail!(
                "Invalid reviews config: {field} pattern \"{pattern}\" contains multiple ** segments which is not supported"
            );
        }

        // For patterns with **, we can't fully validate them with the glob crate
        // since it doesn't support ** the same way. Just do basic validation
        // of any character classes.
        let without_stars = pattern.replace("**", "");
        if (without_stars.contains('[') || without_stars.contains('*'))
            && let Err(e) = glob::Pattern::new(&without_stars.replace('/', ""))
        {
            bail!("Invalid reviews config: {field} pattern \"{pattern}\" is not a valid glob: {e}");
        }
        return Ok(());
    }

    // Validate patterns containing glob metacharacters using glob crate
    if (pattern.contains('*') || pattern.contains('[') || pattern.contains('?'))
        && let Err(e) = glob::Pattern::new(pattern)
    {
        bail!("Invalid reviews config: {field} pattern \"{pattern}\" is not a valid glob: {e}");
    }

    Ok(())
}

/// Reviews specification - handles both boolean and detailed forms
///
/// Allows configuration as either:
/// - Simple boolean: `reviews = true` or `reviews = false`
/// - Detailed config: `[site.reviews]` with options
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum ReviewsSpec {
    /// Simple boolean: reviews = true/false
    Enabled(bool),
    /// Detailed config: [site.reviews] with options
    Config(ReviewsConfig),
}

impl ReviewsSpec {
    /// Check if reviews are enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            ReviewsSpec::Enabled(enabled) => *enabled,
            ReviewsSpec::Config(config) => config.enabled,
        }
    }

    /// Convert to a full ReviewsConfig, applying defaults for simple boolean form
    pub fn to_config(&self) -> ReviewsConfig {
        match self {
            ReviewsSpec::Enabled(enabled) => ReviewsConfig {
                enabled: *enabled,
                ..Default::default()
            },
            ReviewsSpec::Config(config) => config.clone(),
        }
    }
}

impl Default for ReviewsSpec {
    fn default() -> Self {
        ReviewsSpec::Enabled(false)
    }
}

impl ReviewsSpec {
    /// Validate the reviews specification
    pub fn validate(&self) -> Result<()> {
        match self {
            ReviewsSpec::Enabled(_) => Ok(()),
            ReviewsSpec::Config(config) => config.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reviews_spec_simple_true() -> Result<(), serde_json::Error> {
        let spec: ReviewsSpec = serde_json::from_str("true")?;
        assert!(spec.is_enabled());
        let config = spec.to_config();
        assert!(config.enabled);
        assert!(config.is_public());
        assert!(!config.is_anon());
        Ok(())
    }

    #[test]
    fn test_reviews_spec_simple_false() -> Result<(), serde_json::Error> {
        let spec: ReviewsSpec = serde_json::from_str("false")?;
        assert!(!spec.is_enabled());
        Ok(())
    }

    #[test]
    fn test_reviews_spec_detailed() -> Result<(), serde_json::Error> {
        let json = r#"{
            "enabled": true,
            "public": false,
            "anon": true,
            "position": "top-left",
            "types": ["comment"],
            "min-selection": 20,
            "max-selection": 1000,
            "shortcuts": true
        }"#;
        let spec: ReviewsSpec = serde_json::from_str(json)?;
        assert!(spec.is_enabled());

        let config = spec.to_config();
        assert!(!config.is_public());
        assert!(config.is_anon());
        assert_eq!(config.position(), ReviewsPosition::TopLeft);
        assert!(config.allows_comments());
        assert!(!config.allows_suggestions());
        assert_eq!(config.min_selection(), 20);
        assert_eq!(config.max_selection(), 1000);
        assert!(config.shortcuts_enabled());
        Ok(())
    }

    #[test]
    fn test_reviews_position_serialization() -> Result<(), serde_json::Error> {
        assert_eq!(
            serde_json::to_string(&ReviewsPosition::BottomRight)?,
            "\"bottom-right\""
        );
        assert_eq!(
            serde_json::to_string(&ReviewsPosition::TopLeft)?,
            "\"top-left\""
        );
        Ok(())
    }

    #[test]
    fn test_review_type_serialization() -> Result<(), serde_json::Error> {
        assert_eq!(serde_json::to_string(&ReviewType::Comment)?, "\"comment\"");
        assert_eq!(
            serde_json::to_string(&ReviewType::Suggestion)?,
            "\"suggestion\""
        );
        Ok(())
    }

    #[test]
    fn test_validate_min_greater_than_max() {
        let config = ReviewsConfig {
            min_selection: Some(100),
            max_selection: Some(10),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(
            err.to_string()
                .contains("min-selection (100) cannot be greater than max-selection (10)")
        );
    }

    #[test]
    fn test_validate_empty_types() {
        let config = ReviewsConfig {
            types: Some(vec![]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("types cannot be empty"));
    }

    #[test]
    fn test_validate_valid_config() {
        let config = ReviewsConfig {
            min_selection: Some(5),
            max_selection: Some(1000),
            types: Some(vec![ReviewType::Comment]),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_defaults() {
        let config = ReviewsConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_glob_patterns() {
        let config = ReviewsConfig {
            include: Some(vec![
                "docs/**".to_string(),
                "guides/*.md".to_string(),
                "api/v1/**".to_string(),
            ]),
            exclude: Some(vec!["changelog/**".to_string(), "*.tmp".to_string()]),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_glob_pattern() {
        let config = ReviewsConfig {
            include: Some(vec!["docs/[invalid".to_string()]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("include pattern"));
    }

    #[test]
    fn test_validate_multiple_double_star_segments() {
        let config = ReviewsConfig {
            include: Some(vec!["**/docs/**".to_string()]),
            ..Default::default()
        };
        let result = config.validate();
        let err = result.expect_err("should fail validation");
        assert!(err.to_string().contains("multiple ** segments"));
    }
}
