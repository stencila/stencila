//! Configuration for site Content Credentials.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Content Credentials privacy projection profile.
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SiteContentCredentialsProfile {
    /// Public-safe credential metadata.
    #[default]
    Public,

    /// More local detail for internal sharing.
    Private,

    /// Full local detail for controlled archives.
    Full,
}

/// Content Credentials configuration (detailed form), e.g.
/// ```toml
/// [site.content-credentials]
/// profile = "public"
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SiteContentCredentialsConfig {
    /// Whether Content Credentials signing is enabled.
    ///
    /// If omitted in the detailed table form, Content Credentials are enabled.
    pub enabled: Option<bool>,

    /// The privacy/signing projection profile to use.
    ///
    /// Defaults to `public`.
    pub profile: Option<SiteContentCredentialsProfile>,
}

impl SiteContentCredentialsConfig {
    /// Check if Content Credentials signing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    /// Get the effective profile.
    pub fn profile(&self) -> SiteContentCredentialsProfile {
        self.profile.unwrap_or_default()
    }
}

/// Content Credentials specification.
///
/// Allows configuration as:
/// - Simple boolean: `content-credentials = true` or `false`
/// - Profile shorthand: `content-credentials = "public"`
/// - Detailed config: `[site.content-credentials]` with options
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum SiteContentCredentialsSpec {
    /// Simple boolean: content-credentials = true/false
    Enabled(bool),
    /// Profile shorthand: content-credentials = "public"
    Profile(SiteContentCredentialsProfile),
    /// Detailed config: [site.content-credentials] with options
    Config(SiteContentCredentialsConfig),
}

impl SiteContentCredentialsSpec {
    /// Check if Content Credentials signing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.to_config().is_enabled()
    }

    /// Convert to a full config, applying defaults for shorthand forms.
    pub fn to_config(&self) -> SiteContentCredentialsConfig {
        match self {
            SiteContentCredentialsSpec::Enabled(enabled) => SiteContentCredentialsConfig {
                enabled: Some(*enabled),
                ..Default::default()
            },
            SiteContentCredentialsSpec::Profile(profile) => SiteContentCredentialsConfig {
                enabled: Some(true),
                profile: Some(*profile),
            },
            SiteContentCredentialsSpec::Config(config) => config.clone(),
        }
    }
}

impl Default for SiteContentCredentialsSpec {
    fn default() -> Self {
        SiteContentCredentialsSpec::Enabled(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_credentials_spec_simple_true() -> Result<(), serde_json::Error> {
        let spec: SiteContentCredentialsSpec = serde_json::from_str("true")?;
        assert!(spec.is_enabled());
        assert_eq!(
            spec.to_config().profile(),
            SiteContentCredentialsProfile::Public
        );
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_simple_false() -> Result<(), serde_json::Error> {
        let spec: SiteContentCredentialsSpec = serde_json::from_str("false")?;
        assert!(!spec.is_enabled());
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_profile() -> Result<(), serde_json::Error> {
        let spec: SiteContentCredentialsSpec = serde_json::from_str("\"private\"")?;
        assert!(spec.is_enabled());
        assert_eq!(
            spec.to_config().profile(),
            SiteContentCredentialsProfile::Private
        );
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_detailed_profile_only() -> Result<(), serde_json::Error> {
        let json = r#"{"profile": "full"}"#;
        let spec: SiteContentCredentialsSpec = serde_json::from_str(json)?;
        assert!(spec.is_enabled());
        assert_eq!(
            spec.to_config().profile(),
            SiteContentCredentialsProfile::Full
        );
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_detailed_disabled() -> Result<(), serde_json::Error> {
        let json = r#"{"enabled": false, "profile": "full"}"#;
        let spec: SiteContentCredentialsSpec = serde_json::from_str(json)?;
        assert!(!spec.is_enabled());
        assert_eq!(
            spec.to_config().profile(),
            SiteContentCredentialsProfile::Full
        );
        Ok(())
    }
}
