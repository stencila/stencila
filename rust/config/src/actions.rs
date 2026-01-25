//! Configuration for site actions zone
//!
//! The actions zone consolidates multiple floating action buttons (FABs) like
//! reviews and uploads into a single organized area. Supports two modes:
//! - Collapsed: A main FAB that expands on click to reveal action buttons
//! - Expanded: All action buttons always visible

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Position for the actions zone on the page
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ActionsPosition {
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

/// Expansion direction for action buttons
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ActionsDirection {
    /// Vertical stack (default) - buttons expand upward/downward from corner
    #[default]
    Vertical,
    /// Horizontal row - buttons expand left/right from corner
    Horizontal,
}

/// Display mode for the actions zone
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ActionsMode {
    /// Collapsed (default) - main FAB expands on click to reveal action buttons
    #[default]
    Collapsed,
    /// Expanded - all action buttons always visible, no main FAB
    Expanded,
}

/// Actions zone configuration, e.g.
/// ```toml
/// [site.actions]
/// position = "bottom-right"
/// direction = "vertical"
/// mode = "collapsed"
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ActionsConfig {
    /// Position of the actions zone on the page
    ///
    /// Default: bottom-right
    pub position: Option<ActionsPosition>,

    /// Direction for action buttons to expand
    ///
    /// Default: vertical
    pub direction: Option<ActionsDirection>,

    /// Display mode for the actions zone
    ///
    /// Default: collapsed
    pub mode: Option<ActionsMode>,
}

impl ActionsConfig {
    /// Get the effective position (defaults to BottomRight)
    pub fn position(&self) -> ActionsPosition {
        self.position.unwrap_or_default()
    }

    /// Get the effective direction (defaults to Vertical)
    pub fn direction(&self) -> ActionsDirection {
        self.direction.unwrap_or_default()
    }

    /// Get the effective mode (defaults to Collapsed)
    pub fn mode(&self) -> ActionsMode {
        self.mode.unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actions_position_serialization() -> Result<(), serde_json::Error> {
        assert_eq!(
            serde_json::to_string(&ActionsPosition::BottomRight)?,
            "\"bottom-right\""
        );
        assert_eq!(
            serde_json::to_string(&ActionsPosition::TopLeft)?,
            "\"top-left\""
        );
        Ok(())
    }

    #[test]
    fn test_actions_direction_serialization() -> Result<(), serde_json::Error> {
        assert_eq!(
            serde_json::to_string(&ActionsDirection::Vertical)?,
            "\"vertical\""
        );
        assert_eq!(
            serde_json::to_string(&ActionsDirection::Horizontal)?,
            "\"horizontal\""
        );
        Ok(())
    }

    #[test]
    fn test_actions_mode_serialization() -> Result<(), serde_json::Error> {
        assert_eq!(
            serde_json::to_string(&ActionsMode::Collapsed)?,
            "\"collapsed\""
        );
        assert_eq!(
            serde_json::to_string(&ActionsMode::Expanded)?,
            "\"expanded\""
        );
        Ok(())
    }

    #[test]
    fn test_actions_config_defaults() {
        let config = ActionsConfig::default();
        assert_eq!(config.position(), ActionsPosition::BottomRight);
        assert_eq!(config.direction(), ActionsDirection::Vertical);
        assert_eq!(config.mode(), ActionsMode::Collapsed);
    }

    #[test]
    fn test_actions_config_deserialization() -> Result<(), serde_json::Error> {
        let json = r#"{
            "position": "top-left",
            "direction": "horizontal",
            "mode": "expanded"
        }"#;
        let config: ActionsConfig = serde_json::from_str(json)?;
        assert_eq!(config.position(), ActionsPosition::TopLeft);
        assert_eq!(config.direction(), ActionsDirection::Horizontal);
        assert_eq!(config.mode(), ActionsMode::Expanded);
        Ok(())
    }
}
