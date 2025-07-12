//! Unified tool management.
//!
//! This crate provides an interface for managing various development tools
//! including environment managers (e.g. devbox, mise, nix), programming language runtimes
//! (e.g. python, node, r), package managers (e.g. uv, npm), linters (e.g. ruff), and conversion tools
//! (e.g. pandoc, xelatex). It supports automatic tool discovery, installation, version
//! management, and nested environment orchestration.

use common::eyre::{OptionExt, Result};

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
pub use tool::{detect_managers, ToolType};

use crate::collaboration::*;
use crate::conversion::*;
use crate::environments::*;
use crate::execution::*;
use crate::linting::*;
use crate::packages::*;
use crate::tool::Tool;

/// Get a list of tools used by Stencila
pub fn list() -> Vec<Box<dyn Tool>> {
    vec![
        // Environments
        Box::new(Devbox),
        Box::new(Mise),
        Box::new(Nix),
        Box::new(Pixi),
        Box::new(Rig),
        // Packages
        Box::new(Npm),
        Box::new(Uv),
        Box::new(Renv),
        // Execution
        Box::new(Bash),
        Box::new(Node),
        Box::new(Python),
        Box::new(R),
        // Linting
        Box::new(Ruff),
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
            let mut map = common::serde_json::Map::with_capacity(capacity);
            $(
                map.insert($key.to_string(), common::serde_json::json!($value));
            )+
            map
        }
    };
}
