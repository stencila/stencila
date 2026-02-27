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
    /// Usage hint showing required positional args, e.g. `"<ID> <SPEC>..."`.
    /// Empty when the command has no required positional args.
    pub usage_hint: String,
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

    let usage_hint: String = cmd
        .get_arguments()
        .filter(|arg| arg.is_positional() && arg.is_required_set() && !arg.is_hide_set())
        .map(|arg| {
            let name = arg.get_id().as_str().to_uppercase();
            if arg.get_num_args().is_some_and(|r| r.max_values() > 1) {
                format!("<{name}>...")
            } else {
                format!("<{name}>")
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    CliCommandNode {
        name: cmd.get_name().to_string(),
        description: cmd.get_about().map(ToString::to_string).unwrap_or_default(),
        children,
        usage_hint,
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

/// Walk the CLI tree along `path` and return the usage hint for the deepest
/// matching node, but only when the user hasn't provided any extra args
/// beyond the matched command words.
///
/// Returns `None` when the leaf has no required positional args or when extra
/// positional args were already supplied.
#[must_use]
pub fn find_missing_args_hint(tree: &[CliCommandNode], args: &[String]) -> Option<String> {
    let mut nodes = tree;
    let mut depth = 0usize;
    let mut leaf = None;
    for arg in args {
        if let Some(node) = nodes.iter().find(|n| n.name == *arg) {
            leaf = Some(node);
            nodes = &node.children;
            depth += 1;
        } else {
            break;
        }
    }
    let node = leaf?;
    // User already provided args beyond the matched command path
    if args.len() > depth {
        return None;
    }
    if node.usage_hint.is_empty() {
        return None;
    }
    Some(node.usage_hint.clone())
}

/// A small CLI tree fixture for tests across the crate.
#[cfg(test)]
pub(crate) fn test_cli_tree() -> Vec<CliCommandNode> {
    vec![
        CliCommandNode {
            name: "skills".to_string(),
            description: "Manage skills".to_string(),
            usage_hint: String::new(),
            children: vec![
                CliCommandNode {
                    name: "list".to_string(),
                    description: "List skills".to_string(),
                    usage_hint: String::new(),
                    children: vec![],
                },
                CliCommandNode {
                    name: "show".to_string(),
                    description: "Show a skill".to_string(),
                    usage_hint: "<NAME>".to_string(),
                    children: vec![],
                },
            ],
        },
        CliCommandNode {
            name: "agents".to_string(),
            description: "Manage agents".to_string(),
            usage_hint: String::new(),
            children: vec![CliCommandNode {
                name: "list".to_string(),
                description: "List agents".to_string(),
                usage_hint: String::new(),
                children: vec![],
            }],
        },
        CliCommandNode {
            name: "models".to_string(),
            description: "Manage AI models".to_string(),
            usage_hint: String::new(),
            children: vec![],
        },
        CliCommandNode {
            name: "formats".to_string(),
            description: "Manage formats".to_string(),
            usage_hint: String::new(),
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
                    .subcommand(
                        clap::Command::new("show")
                            .about("Show a skill")
                            .arg(clap::Arg::new("name").required(true)),
                    )
                    .subcommand(
                        clap::Command::new("internal")
                            .about("Internal command")
                            .hide(true),
                    ),
            )
            .subcommand(clap::Command::new("models").about("Manage AI models"))
            .subcommand(
                clap::Command::new("mcp")
                    .about("Manage MCP servers")
                    .subcommand(clap::Command::new("list").about("List servers"))
                    .subcommand(
                        clap::Command::new("add")
                            .about("Add a server")
                            .arg(clap::Arg::new("id").required(true))
                            .arg(clap::Arg::new("spec").required(true).num_args(1..)),
                    ),
            )
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

    #[test]
    fn usage_hint_extracted_from_clap() {
        let tree = build_command_tree(&synthetic_cli());
        let skills = tree.iter().find(|n| n.name == "skills").unwrap();
        // "list" has no required positional args
        let list = skills.children.iter().find(|n| n.name == "list").unwrap();
        assert!(list.usage_hint.is_empty());
        // "show" has a required <name> arg
        let show = skills.children.iter().find(|n| n.name == "show").unwrap();
        assert_eq!(show.usage_hint, "<NAME>");
    }

    #[test]
    fn usage_hint_variadic_args() {
        let tree = build_command_tree(&synthetic_cli());
        let mcp = tree.iter().find(|n| n.name == "mcp").unwrap();
        let add = mcp.children.iter().find(|n| n.name == "add").unwrap();
        assert_eq!(add.usage_hint, "<ID> <SPEC>...");
    }

    #[test]
    fn find_missing_args_hint_returns_hint() {
        let tree = test_cli_tree();
        let hint = find_missing_args_hint(&tree, &["skills".into(), "show".into()]);
        assert_eq!(hint, Some("<NAME>".to_string()));
    }

    #[test]
    fn find_missing_args_hint_none_when_no_required() {
        let tree = test_cli_tree();
        let hint = find_missing_args_hint(&tree, &["skills".into(), "list".into()]);
        assert_eq!(hint, None);
    }

    #[test]
    fn find_missing_args_hint_none_when_extra_args() {
        let tree = test_cli_tree();
        let hint = find_missing_args_hint(&tree, &["skills".into(), "show".into(), "foo".into()]);
        assert_eq!(hint, None);
    }
}
