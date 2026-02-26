//! Configuration for agent selection.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// How Stencila should be attributed in git commits made by agents.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CommitAttribution {
    /// Set Stencila as the commit author (`GIT_AUTHOR_*`).
    Author,

    /// Keep normal author/committer identity and add a Stencila co-author trailer.
    #[default]
    CoAuthor,

    /// Set Stencila as the commit committer (`GIT_COMMITTER_*`).
    Committer,

    /// Do not mention Stencila in commit attribution.
    None,
}

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

    /// How Stencila should be attributed in git commits made by agents.
    ///
    /// Supported values:
    /// - `"author"`: Stencila is set as commit author.
    /// - `"co-author"`: Stencila is added as commit co-author (default).
    /// - `"committer"`: Stencila is set as commit committer.
    /// - `"none"`: Stencila is not mentioned in attribution.
    ///
    /// ```toml
    /// [agents]
    /// commit_attribution = "co-author"
    /// ```
    pub commit_attribution: Option<CommitAttribution>,
}
