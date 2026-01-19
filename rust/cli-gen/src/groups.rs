//! Command group definitions for CLI navigation
//!
//! Defines logical groupings of top-level CLI commands for documentation navigation.
//! Order matches the Command enum in rust/cli/src/cli.rs.

/// A group of related CLI commands
pub struct CommandGroup {
    pub label: &'static str,
    pub commands: &'static [&'static str],
}

/// Groups of top-level CLI commands
///
/// Order matches the Command enum definition in `rust/cli/src/cli.rs`.
/// Edit this list to change how commands are grouped in documentation navigation.
pub const COMMAND_GROUPS: &[CommandGroup] = &[
    CommandGroup {
        label: "Create & Setup",
        commands: &["new", "init", "config"],
    },
    CommandGroup {
        label: "Document Management",
        commands: &["status", "move", "track", "untrack", "clean"],
    },
    CommandGroup {
        label: "Format & Sync",
        commands: &[
            "convert", "merge", "sync", "push", "pull", "watch", "unwatch",
        ],
    },
    CommandGroup {
        label: "Processing",
        commands: &["compile", "lint", "execute", "render", "query"],
    },
    CommandGroup {
        label: "View & Publish",
        commands: &["open", "publish", "demo"],
    },
    CommandGroup {
        label: "Resources",
        commands: &[
            "outputs", "db", "prompts", "models", "kernels", "linters", "formats", "themes",
            "secrets", "tools",
        ],
    },
    CommandGroup {
        label: "Server",
        commands: &["serve", "snap", "lsp"],
    },
    CommandGroup {
        label: "Cloud",
        commands: &["cloud", "site", "signin", "signout", "logs"],
    },
    CommandGroup {
        label: "Maintenance",
        commands: &["upgrade", "uninstall"],
    },
];
