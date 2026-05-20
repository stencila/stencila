//! Configuration for Content Credentials.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Content Credentials privacy projection profile.
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ContentCredentialsProfile {
    /// Public-safe credential metadata.
    #[default]
    Public,

    /// More local detail for internal sharing.
    Private,

    /// Full local detail for controlled archives.
    Full,
}

/// Content Credentials signing backend.
#[derive(
    Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, JsonSchema, strum::Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ContentCredentialsSigner {
    /// Use Cloud signing when available, otherwise fall back to local signing.
    #[default]
    Auto,

    /// Use Stencila Cloud's signing service.
    Cloud,

    /// Use the local self-signed signing identity.
    Local,
}

/// Content Credentials configuration (detailed form), e.g.
/// ```toml
/// [content-credentials]
/// enabled = true
/// profile = "public"
/// signer = "auto"
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ContentCredentialsConfig {
    /// Whether Content Credentials signing is enabled.
    ///
    /// If omitted in the detailed table form, Content Credentials are enabled.
    pub enabled: Option<bool>,

    /// The privacy/signing projection profile to use.
    ///
    /// Defaults to `public`.
    pub profile: Option<ContentCredentialsProfile>,

    /// The signing backend to use.
    ///
    /// Defaults to `auto`.
    pub signer: Option<ContentCredentialsSigner>,
}

impl ContentCredentialsConfig {
    /// Check if Content Credentials signing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    /// Get the effective profile.
    pub fn profile(&self) -> ContentCredentialsProfile {
        self.profile.unwrap_or_default()
    }

    /// Get the effective signing backend.
    pub fn signer(&self) -> ContentCredentialsSigner {
        self.signer.unwrap_or_default()
    }

    /// Merge an override config into this config.
    ///
    /// Fields that are absent in `override_config` keep the value from `self`.
    #[must_use]
    pub fn merge_override(&self, override_config: &Self) -> Self {
        Self {
            enabled: override_config.enabled.or(self.enabled),
            profile: override_config.profile.or(self.profile),
            signer: override_config.signer.or(self.signer),
        }
    }
}

/// Content Credentials specification.
///
/// Allows configuration as:
/// - Simple boolean: `content-credentials = true` or `false`
/// - Profile shorthand: `content-credentials = "public"`
/// - Detailed config: `[content-credentials]` with options
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum ContentCredentialsSpec {
    /// Simple boolean: content-credentials = true/false
    Enabled(bool),
    /// Profile shorthand: content-credentials = "public"
    Profile(ContentCredentialsProfile),
    /// Detailed config: [content-credentials] with options
    Config(ContentCredentialsConfig),
}

impl ContentCredentialsSpec {
    /// Check if Content Credentials signing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.to_config().is_enabled()
    }

    /// Convert to a full config, applying defaults for shorthand forms.
    pub fn to_config(&self) -> ContentCredentialsConfig {
        match self {
            ContentCredentialsSpec::Enabled(enabled) => ContentCredentialsConfig {
                enabled: Some(*enabled),
                ..Default::default()
            },
            ContentCredentialsSpec::Profile(profile) => ContentCredentialsConfig {
                enabled: Some(true),
                profile: Some(*profile),
                ..Default::default()
            },
            ContentCredentialsSpec::Config(config) => config.clone(),
        }
    }
}

impl Default for ContentCredentialsSpec {
    fn default() -> Self {
        ContentCredentialsSpec::Enabled(false)
    }
}

/// Compatibility alias for the previous site-specific profile name.
pub type SiteContentCredentialsProfile = ContentCredentialsProfile;

/// Compatibility alias for the previous site-specific signer name.
pub type SiteContentCredentialsSigner = ContentCredentialsSigner;

/// Compatibility alias for the previous site-specific config name.
pub type SiteContentCredentialsConfig = ContentCredentialsConfig;

/// Compatibility alias for the previous site-specific spec name.
pub type SiteContentCredentialsSpec = ContentCredentialsSpec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_credentials_spec_simple_true() -> Result<(), serde_json::Error> {
        let spec: ContentCredentialsSpec = serde_json::from_str("true")?;
        assert!(spec.is_enabled());
        assert_eq!(
            spec.to_config().profile(),
            ContentCredentialsProfile::Public
        );
        assert_eq!(spec.to_config().signer(), ContentCredentialsSigner::Auto);
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_simple_false() -> Result<(), serde_json::Error> {
        let spec: ContentCredentialsSpec = serde_json::from_str("false")?;
        assert!(!spec.is_enabled());
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_profile() -> Result<(), serde_json::Error> {
        let spec: ContentCredentialsSpec = serde_json::from_str("\"private\"")?;
        assert!(spec.is_enabled());
        assert_eq!(
            spec.to_config().profile(),
            ContentCredentialsProfile::Private
        );
        assert_eq!(spec.to_config().signer(), ContentCredentialsSigner::Auto);
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_detailed_profile_only() -> Result<(), serde_json::Error> {
        let json = r#"{"profile": "full"}"#;
        let spec: ContentCredentialsSpec = serde_json::from_str(json)?;
        assert!(spec.is_enabled());
        assert_eq!(spec.to_config().profile(), ContentCredentialsProfile::Full);
        assert_eq!(spec.to_config().signer(), ContentCredentialsSigner::Auto);
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_detailed_auto_signer() -> Result<(), serde_json::Error> {
        let json = r#"{"signer": "auto"}"#;
        let spec: ContentCredentialsSpec = serde_json::from_str(json)?;
        let config = spec.to_config();
        assert!(config.is_enabled());
        assert_eq!(config.signer(), ContentCredentialsSigner::Auto);
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_detailed_cloud_signer() -> Result<(), serde_json::Error> {
        let json = r#"{"profile": "public", "signer": "cloud"}"#;
        let spec: ContentCredentialsSpec = serde_json::from_str(json)?;
        let config = spec.to_config();
        assert!(config.is_enabled());
        assert_eq!(config.profile(), ContentCredentialsProfile::Public);
        assert_eq!(config.signer(), ContentCredentialsSigner::Cloud);
        Ok(())
    }

    #[test]
    fn test_content_credentials_spec_detailed_disabled() -> Result<(), serde_json::Error> {
        let json = r#"{"enabled": false, "profile": "full"}"#;
        let spec: ContentCredentialsSpec = serde_json::from_str(json)?;
        assert!(!spec.is_enabled());
        assert_eq!(spec.to_config().profile(), ContentCredentialsProfile::Full);
        Ok(())
    }

    #[test]
    fn test_content_credentials_config_merge_override() {
        let root = ContentCredentialsConfig {
            enabled: Some(true),
            profile: Some(ContentCredentialsProfile::Private),
            signer: Some(ContentCredentialsSigner::Cloud),
        };
        let site = ContentCredentialsConfig {
            enabled: None,
            profile: Some(ContentCredentialsProfile::Public),
            signer: None,
        };

        let merged = root.merge_override(&site);

        assert!(merged.is_enabled());
        assert_eq!(merged.profile(), ContentCredentialsProfile::Public);
        assert_eq!(merged.signer(), ContentCredentialsSigner::Cloud);
    }
}
