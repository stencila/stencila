use std::{collections::HashMap, path::PathBuf, process::Command};

use common::{
    clap::{self, ValueEnum},
    once_cell::sync::Lazy,
    regex::Regex,
    serde::Serialize,
    strum::Display,
};
use mcp_types::{Tool as McpTool, ToolInputSchema as McpToolInputSchema};
pub use semver::{Version, VersionReq};
use which::which;

use collaboration::*;
use conversion::*;
use environments::*;
use execution::*;
use linting::*;
use packages::*;

pub mod cli;
mod collaboration;
mod conversion;
mod environments;
mod execution;
mod linting;
mod packages;

/// Get a list of tools used by Stencila
pub fn list() -> Vec<Box<dyn Tool>> {
    vec![
        // Environments
        Box::new(Asdf) as Box<dyn Tool>,
        Box::new(Devbox) as Box<dyn Tool>,
        Box::new(Mise) as Box<dyn Tool>,
        Box::new(Pixi) as Box<dyn Tool>,
        // Packages
        Box::new(Npm) as Box<dyn Tool>,
        Box::new(Uv) as Box<dyn Tool>,
        // Execution
        Box::new(Bash) as Box<dyn Tool>,
        Box::new(Node) as Box<dyn Tool>,
        Box::new(Python) as Box<dyn Tool>,
        Box::new(R) as Box<dyn Tool>,
        // Linting
        Box::new(Ruff) as Box<dyn Tool>,
        // Conversion
        Box::new(Pandoc) as Box<dyn Tool>,
        Box::new(Xelatex) as Box<dyn Tool>,
        // Collaboration
        Box::new(Git) as Box<dyn Tool>,
    ]
}

/// Get a tool by name
pub fn get(name: &str) -> Option<Box<dyn Tool>> {
    list().into_iter().find(|tool| tool.name() == name)
}

/// The type of a kernel
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, ValueEnum)]
#[serde(crate = "common::serde")]
#[strum(serialize_all = "lowercase")]
pub enum ToolType {
    Collaboration,
    Conversion,
    Environments,
    Execution,
    Linting,
    Packages,
}

pub trait Tool {
    /// The name of the tool
    fn name(&self) -> &'static str;

    /// A URL for the tool
    fn url(&self) -> &'static str;

    /// A description of the tool
    fn description(&self) -> &'static str;

    /// The type of the tool
    fn r#type(&self) -> ToolType;

    /// The name of the tool's executable used by Stencila
    ///
    /// Used to search for the tool on the $PATH.
    /// By default, the same as `self.name()`.
    fn executable_name(&self) -> &'static str {
        self.name()
    }

    /// The path to the tool (if any)
    ///
    /// Searches on the $PATH for tool, using its name. If an environment
    /// tool such as `mise` or `devbox` is available then the path
    /// should be within any environments defined by them.
    fn path(&self) -> Option<PathBuf> {
        which(self.executable_name()).ok()
    }

    /// The version required by Stencila
    ///
    /// Defaults to any version and should be overridden if a
    fn version_required(&self) -> VersionReq {
        VersionReq::STAR
    }

    /// The version available (if any)
    ///
    /// Defaults to calling the discovered binary with the `--version` argument
    /// and then parsing it into a version.
    ///
    /// Note that Stencila uses (and returns from this function) semantic version numbers
    /// which muse have major.minor.patch components. Therefore this function will add
    /// minor and patch versions of 0 is necessary. As such the version returned here
    /// may not exactly match the string returned by the tool.
    fn version_available(&self) -> Option<Version> {
        let path = self.path()?;

        let unknown = Version::new(0, 0, 0);

        let Ok(output) = Command::new(path).arg("--version").output() else {
            return Some(unknown);
        };

        let output = String::from_utf8_lossy(&output.stdout);

        let Some(line) = output.lines().next() else {
            return Some(unknown);
        };

        // Some tools have a version string like `3.141592653-2.6` or `2025-05-09` so
        // replace hyphens with dots so that we can extract as many parts as possible.
        let line = line.replace("-", ".");

        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(\d+)(?:\.(\d+))?(?:\.(\d+))?").expect("invalid regex"));

        let Some(captures) = REGEX.captures(&line) else {
            return Some(unknown);
        };

        let part = |index| {
            captures
                .get(index)
                .map(|m| m.as_str())
                .and_then(|m| m.parse().ok())
                .unwrap_or_default()
        };
        let major = part(1);
        let minor = part(2);
        let patch = part(3);

        Some(Version::new(major, minor, patch))
    }

    /// Create a Model Context Protocol (MCP) specification for the tool
    ///
    /// This default implementation simply exposes the tool with an input schema
    /// as an array of  
    fn mcp_tool(&self) -> McpTool {
        let version = self
            .version_available()
            .map(|version| ["(version ", &version.to_string(), ")"].concat())
            .unwrap_or_else(|| "(unavailable)".to_string());

        McpTool {
            name: self.name().into(),
            description: Some([self.description(), " ", self.url(), " ", &version].concat()),
            input_schema: self.mcp_tool_inputs(),
        }
    }

    /// Create a Model Context Protocol (MCP) specification for the tool
    ///
    /// This default implementation simply exposes the tool with an input schema
    /// as an array of  
    fn mcp_tool_inputs(&self) -> McpToolInputSchema {
        McpToolInputSchema {
            type_: "object".into(),
            required: vec!["arguments".into()],
            properties: HashMap::from([(
                "arguments".into(),
                json_map! {
                    "type" => "array",
                    "items"  => "string"
                },
            )]),
        }
    }
}

/// Macro to create a [`serde_json::Map`] needed within MCP tool definition
#[macro_export]
macro_rules! json_map {
    // Count the number of key-value pairs at compile time
    (@count) => { 0 };
    (@count $key:expr => $value:expr) => { 1 };
    (@count $key:expr => $value:expr, $($rest:tt)*) => {
        1 + json_map!(@count $($rest)*)
    };

    // Empty map
    {} => {
        serde_json::Map::new()
    };

    // Map with pre-allocated capacity
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let capacity = json_map!(@count $($key => $value),+);
            let mut map = common::serde_json::Map::with_capacity(capacity);
            $(
                map.insert($key.to_string(), common::serde_json::json!($value));
            )+
            map
        }
    };
}
