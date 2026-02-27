use std::sync::Arc;

/// A node in the CLI command tree, built from clap introspection.
#[derive(Debug, Clone)]
pub struct CliCommandNode {
    /// The command name (e.g. "skills", "list")
    pub name: String,
    /// Short description from clap's `about`
    pub description: String,
    /// Child subcommands
    pub children: Vec<CliCommandNode>,
}

/// Top-level CLI commands exposed as TUI slash commands.
///
/// New CLI commands default to hidden in TUI until explicitly added here.
const ALLOWLIST: &[&str] = &[
    "skills",
    "models",
    "kernels",
    "agents",
    "workflows",
    "formats",
    "themes",
    "linters",
    "tools",
    "mcp",
    "secrets",
    "config",
    "auth",
];

/// Build the command tree from the top-level clap `Command`.
///
/// Only includes commands present in the allowlist at the top level.
/// Hidden commands (per clap's `is_hide_set()`) are excluded at every level.
/// Children are included recursively without further allowlist checks.
pub fn build_command_tree(cli_command: &clap::Command) -> Vec<CliCommandNode> {
    cli_command
        .get_subcommands()
        .filter(|cmd| !cmd.is_hide_set() && ALLOWLIST.contains(&cmd.get_name()))
        .map(build_node)
        .collect()
}

fn build_node(cmd: &clap::Command) -> CliCommandNode {
    let children: Vec<CliCommandNode> = cmd
        .get_subcommands()
        .filter(|sub| !sub.is_hide_set())
        .map(build_node)
        .collect();

    CliCommandNode {
        name: cmd.get_name().to_string(),
        description: cmd.get_about().map(ToString::to_string).unwrap_or_default(),
        children,
    }
}

/// Wrap the tree in an `Arc` for sharing with autocomplete state.
#[must_use]
pub fn arc_tree(tree: Vec<CliCommandNode>) -> Arc<Vec<CliCommandNode>> {
    Arc::new(tree)
}

/// Return top-level CLI nodes that do not shadow a built-in slash command.
#[must_use]
pub fn visible_top_level(tree: &[CliCommandNode]) -> Vec<&CliCommandNode> {
    tree.iter()
        .filter(|node| !crate::commands::SlashCommand::shadows_builtin(&node.name))
        .collect()
}

/// A small CLI tree fixture for tests across the crate.
#[cfg(test)]
pub(crate) fn test_cli_tree() -> Vec<CliCommandNode> {
    vec![
        CliCommandNode {
            name: "skills".to_string(),
            description: "Manage skills".to_string(),
            children: vec![
                CliCommandNode {
                    name: "list".to_string(),
                    description: "List skills".to_string(),
                    children: vec![],
                },
                CliCommandNode {
                    name: "show".to_string(),
                    description: "Show a skill".to_string(),
                    children: vec![],
                },
            ],
        },
        CliCommandNode {
            name: "agents".to_string(),
            description: "Manage agents".to_string(),
            children: vec![CliCommandNode {
                name: "list".to_string(),
                description: "List agents".to_string(),
                children: vec![],
            }],
        },
        CliCommandNode {
            name: "models".to_string(),
            description: "Manage AI models".to_string(),
            children: vec![],
        },
        CliCommandNode {
            name: "formats".to_string(),
            description: "Manage formats".to_string(),
            children: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_cli() -> clap::Command {
        clap::Command::new("stencila")
            .subcommand(
                clap::Command::new("skills")
                    .about("Manage agent skills")
                    .subcommand(clap::Command::new("list").about("List skills"))
                    .subcommand(clap::Command::new("show").about("Show a skill"))
                    .subcommand(
                        clap::Command::new("internal")
                            .about("Internal command")
                            .hide(true),
                    ),
            )
            .subcommand(clap::Command::new("models").about("Manage AI models"))
            .subcommand(clap::Command::new("convert").about("Convert documents"))
            .subcommand(
                clap::Command::new("hidden-cmd")
                    .about("Should be hidden")
                    .hide(true),
            )
    }

    #[test]
    fn builds_only_allowlisted_commands() {
        let tree = build_command_tree(&synthetic_cli());
        let names: Vec<&str> = tree.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"skills"));
        assert!(names.contains(&"models"));
        // "convert" is not in the allowlist
        assert!(!names.contains(&"convert"));
        // Hidden commands are excluded
        assert!(!names.contains(&"hidden-cmd"));
    }

    #[test]
    fn includes_children_recursively() {
        let tree = build_command_tree(&synthetic_cli());
        let skills = tree
            .iter()
            .find(|n| n.name == "skills")
            .expect("should be present");
        let child_names: Vec<&str> = skills.children.iter().map(|n| n.name.as_str()).collect();
        assert!(child_names.contains(&"list"));
        assert!(child_names.contains(&"show"));
        // Hidden child excluded
        assert!(!child_names.contains(&"internal"));
    }

    #[test]
    fn descriptions_populated() {
        let tree = build_command_tree(&synthetic_cli());
        let models = tree
            .iter()
            .find(|n| n.name == "models")
            .expect("should be present");
        assert_eq!(models.description, "Manage AI models");
    }

    #[test]
    fn empty_tree_when_no_matches() {
        let cmd = clap::Command::new("test")
            .subcommand(clap::Command::new("convert").about("Not in allowlist"));
        let tree = build_command_tree(&cmd);
        assert!(tree.is_empty());
    }
}
