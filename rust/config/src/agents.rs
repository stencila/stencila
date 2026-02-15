//! Configuration for agent selection.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Agents configuration.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentsConfig {
    /// The name of the default agent to use when no agent is specified.
    ///
    /// When set, the TUI and other callers that request the "default" agent
    /// will use this agent instead.
    ///
    /// ```toml
    /// [agents]
    /// default = "code-engineer"
    /// ```
    pub default: Option<String>,
}
