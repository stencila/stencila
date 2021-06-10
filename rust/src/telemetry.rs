///! A module for functions and configuration related to telemetry.
///!
///! Currently this module just has Sentry related settings but
///! in the future may also include functionality around OpenTelemetry
///! or similar for monitoring of worker processes.

#[cfg(feature = "config")]
pub mod config {
    use defaults::Defaults;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    /// Telemetry settings for Stencila CLI
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default)]
    #[schemars(deny_unknown_fields)]
    pub struct TelemetryCLIConfig {
        /// Whether to send error reports to Sentry. Default is false.
        #[def = "false"]
        pub sentry: bool,
    }

    /// Telemetry settings for Stencila Desktop
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default)]
    #[schemars(deny_unknown_fields)]
    pub struct TelemetryDesktopConfig {
        /// Whether to send error reports to Sentry. Default is false.
        #[def = "false"]
        pub sentry: bool,
    }

    /// Telemetry
    ///
    /// Configuration settings for telemetry
    #[derive(Debug, Default, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default)]
    #[schemars(deny_unknown_fields)]
    pub struct TelemetryConfig {
        pub cli: TelemetryCLIConfig,
        pub desktop: TelemetryDesktopConfig,
    }
}
