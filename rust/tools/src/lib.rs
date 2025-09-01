//! Unified tool management.
//!
//! This crate provides an interface for managing various development tools
//! including environment managers (e.g. devbox, mise, nix), programming language runtimes
//! (e.g. python, node, r), package managers (e.g. uv, npm), linters (e.g. ruff), and conversion tools
//! (e.g. pandoc, xelatex). It supports automatic tool discovery, installation, version
//! management, and nested environment orchestration.

use eyre::{OptionExt, Result};

pub mod cli;
mod collaboration;
mod command;
mod conversion;
mod environments;
mod execution;
mod linting;
mod packages;
mod tool;

/// Re-exports for consuming crates
pub use command::{AsyncToolCommand, ToolCommand, ToolStdio};
pub use semver::{Version, VersionReq};

// Export all tools for direct use
pub use collaboration::*;
pub use conversion::*;
pub use environments::*;
pub use execution::*;
pub use linting::*;
pub use packages::*;
pub use tool::{Tool, ToolType, detect_managers};

/// Get a list of tools used by Stencila
pub fn list() -> Vec<Box<dyn Tool>> {
    vec![
        // Environments
        Box::new(Mise),
        Box::new(Devbox),
        Box::new(Nix),
        Box::new(Pixi),
        Box::new(Apt),
        // Packages
        Box::new(Npm),
        Box::new(Npx),
        Box::new(Uv),
        Box::new(Rig),
        Box::new(Renv),
        // Execution
        Box::new(Bash),
        Box::new(Node),
        Box::new(Python),
        Box::new(R),
        // Linting
        Box::new(Ruff),
        Box::new(Pyright),
        Box::new(StyleR),
        Box::new(LintR),
        // Conversion
        Box::new(Agg),
        Box::new(MarkerPdf),
        Box::new(MinerU),
        Box::new(Pandoc),
        Box::new(Xelatex),
        // Collaboration
        Box::new(Git),
    ]
}

/// Get a tool by name
pub fn get(name: &str) -> Option<Box<dyn Tool>> {
    list().into_iter().find(|tool| tool.name() == name)
}

/// Find out if a tool is installed in the current environment
///
/// Errors if the tool is unknown.
pub fn is_installed(name: &str) -> Result<bool> {
    let tool = get(name).ok_or_eyre("Unknown tool")?;
    Ok(tool::is_installed(tool.as_ref()))
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
            let mut map = serde_json::Map::with_capacity(capacity);
            $(
                map.insert($key.to_string(), serde_json::json!($value));
            )+
            map
        }
    };
}
