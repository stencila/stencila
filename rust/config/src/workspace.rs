use eyre::{Result, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{WATCH_ID_REGEX, WORKSPACE_ID_REGEX};

/// Configuration for a Stencila Cloud workspace.
///
/// Workspaces are the primary entity in Stencila Cloud, representing a
/// GitHub repository. Sites and watches are scoped under workspaces, e.g.
///
/// ```toml
/// [workspace]
/// id = "ws3x9k2m7fab"
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct WorkspaceConfig {
    /// The workspace public ID from Stencila Cloud.
    ///
    /// A 12-character string: "ws" prefix followed by 10 lowercase alphanumeric
    /// characters (e.g., "ws3x9k2m7fab").
    ///
    /// This is automatically assigned when a workspace is created via
    /// `stencila site create` or when pushing to a site for the first time.
    /// The workspace ID is derived from the GitHub repository URL.
    #[schemars(regex(pattern = r"^ws[a-z0-9]{10}$"))]
    pub id: Option<String>,

    /// The workspace watch ID from Stencila Cloud.
    ///
    /// A 12-character string: "wa" prefix followed by 10 lowercase alphanumeric
    /// characters (e.g., "wa7x2k9m3fab").
    ///
    /// This is set when `stencila watch` is run without a file path to enable
    /// workspace-level watching. When enabled, `update.sh` is run on each git push.
    #[schemars(regex(pattern = r"^wa[a-z0-9]{10}$"))]
    pub watch: Option<String>,
}

impl WorkspaceConfig {
    /// Validate the workspace configuration
    pub fn validate(&self) -> Result<()> {
        if let Some(id) = &self.id
            && !WORKSPACE_ID_REGEX.is_match(id)
        {
            bail!(
                "Invalid workspace ID `{id}`: must match pattern 'ws' followed by 10 lowercase alphanumeric characters (e.g., 'ws3x9k2m7fab')"
            );
        }
        if let Some(watch) = &self.watch
            && !WATCH_ID_REGEX.is_match(watch)
        {
            bail!(
                "Invalid watch ID `{watch}`: must match pattern 'wa' followed by 10 lowercase alphanumeric characters (e.g., 'wa7x2k9m3fab')"
            );
        }
        Ok(())
    }
}
