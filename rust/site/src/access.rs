//! Access index generation
//!
//! Generates `_access.json` for sites with access restrictions configured.
//! This file is used by the Stencila Cloud worker to enforce route access.

use std::{collections::BTreeMap, path::Path};

use eyre::Result;
use serde::{Deserialize, Serialize};
use stencila_config::{AccessLevel, SiteAccessConfig};
use tokio::fs::write;

/// Access index structure written to `_access.json`
///
/// Uses `BTreeMap` instead of `HashMap` to ensure deterministic output order,
/// avoiding noisy diffs and cache churn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessIndex {
    /// Default access level for routes not explicitly configured
    pub default: AccessLevel,

    /// Route path to access level mappings (sorted by key for deterministic output)
    pub routes: BTreeMap<String, AccessLevel>,
}

impl From<&SiteAccessConfig> for AccessIndex {
    fn from(config: &SiteAccessConfig) -> Self {
        Self {
            default: config.default,
            // Convert HashMap to BTreeMap for sorted output
            routes: config.routes.iter().map(|(k, v)| (k.clone(), *v)).collect(),
        }
    }
}

/// Generate access index for a site
///
/// Creates `_access.json` at the output root containing route access configuration.
/// Only generates the file if access restrictions are configured.
///
/// # Arguments
/// * `config` - The site access configuration
/// * `output_dir` - The output directory where the site is rendered
///
/// # Returns
/// * `Ok(true)` if the file was written
/// * `Ok(false)` if no restrictions were configured (file not written)
pub async fn generate_access_index(config: &SiteAccessConfig, output_dir: &Path) -> Result<bool> {
    // Only generate if there are actual restrictions
    if !config.has_restrictions() {
        return Ok(false);
    }

    let index = AccessIndex::from(config);
    let json = serde_json::to_string_pretty(&index)?;
    let path = output_dir.join("_access.json");
    write(&path, json).await?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio::fs::read_to_string;

    #[tokio::test]
    async fn test_generate_access_index_with_restrictions() -> Result<()> {
        let output_dir = TempDir::new()?;

        let mut routes = HashMap::new();
        routes.insert("/data/".to_string(), AccessLevel::Password);
        routes.insert("/internal/".to_string(), AccessLevel::Team);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        let generated = generate_access_index(&config, output_dir.path()).await?;
        assert!(generated);

        // Verify file was written
        let path = output_dir.path().join("_access.json");
        assert!(path.exists());

        // Verify content
        let content = read_to_string(&path).await?;
        let parsed: AccessIndex = serde_json::from_str(&content)?;
        assert_eq!(parsed.default, AccessLevel::Public);
        assert_eq!(parsed.routes.get("/data/"), Some(&AccessLevel::Password));
        assert_eq!(parsed.routes.get("/internal/"), Some(&AccessLevel::Team));

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_access_index_no_restrictions() -> Result<()> {
        let output_dir = TempDir::new()?;

        // Default config with no restrictions
        let config = SiteAccessConfig::default();

        let generated = generate_access_index(&config, output_dir.path()).await?;
        assert!(!generated);

        // Verify file was NOT written
        let path = output_dir.path().join("_access.json");
        assert!(!path.exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_access_index_default_restriction() -> Result<()> {
        let output_dir = TempDir::new()?;

        // Non-public default counts as a restriction
        let config = SiteAccessConfig {
            default: AccessLevel::Password,
            routes: HashMap::new(),
        };

        let generated = generate_access_index(&config, output_dir.path()).await?;
        assert!(generated);

        // Verify file was written
        let path = output_dir.path().join("_access.json");
        assert!(path.exists());

        let content = read_to_string(&path).await?;
        let parsed: AccessIndex = serde_json::from_str(&content)?;
        assert_eq!(parsed.default, AccessLevel::Password);

        Ok(())
    }
}
