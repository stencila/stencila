//! Unified tool management.
//!
//! This crate provides an interface for managing various development tools
//! including environment managers (e.g. devbox, mise, nix), programming language runtimes
//! (e.g. python, node, r), package managers (e.g. uv, npm), linters (e.g. ruff), and conversion tools
//! (e.g. pandoc, xelatex). It supports automatic tool discovery, installation, version
//! management, and nested environment orchestration.

pub mod cli;
mod collaboration;
mod command;
mod conversion;
mod environments;
mod execution;
mod linting;
mod package;
mod packages;
mod tool;

/// Re-exports for consuming crates
pub use command::{AsyncToolCommand, ToolCommand, ToolStdio};
pub use semver::{Version, VersionReq};
pub use tool::{detect_managers, is_installed, ToolType};

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
