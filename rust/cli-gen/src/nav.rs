//! Generate navigation YAML for CLI documentation
//!
//! Creates a `_nav.yaml` file that controls the ordering and grouping of CLI
//! commands in the documentation sidebar navigation, including nested subcommands.

use crate::extract::CommandDoc;
use crate::groups::COMMAND_GROUPS;

/// Generate the `_nav.yaml` content for CLI documentation
///
/// The generated YAML file is used by the site builder to override
/// alphabetical ordering of navigation items at all nesting levels.
pub fn generate_nav_yaml(root: &CommandDoc) -> String {
    let mut output = String::new();
    output.push_str("# Auto-generated navigation for CLI docs\n");
    output.push_str("# Edit rust/cli-gen/src/groups.rs to change top-level grouping\n\n");
    output.push_str("items:\n");

    // Build a map from command name to CommandDoc for quick lookup
    let subcommand_map: std::collections::HashMap<&str, &CommandDoc> = root
        .subcommands
        .iter()
        .map(|cmd| (cmd.path.last().unwrap().as_str(), cmd))
        .collect();

    for group in COMMAND_GROUPS {
        output.push_str(&format!("  - label: \"{}\"\n", group.label));
        output.push_str("    children:\n");
        for cmd_name in group.commands {
            if let Some(cmd) = subcommand_map.get(cmd_name) {
                write_command_yaml(&mut output, cmd, 6);
            } else {
                // Simple command without subcommands
                output.push_str(&format!("      - \"{cmd_name}\"\n"));
            }
        }
        output.push('\n');
    }

    output
}

/// Write a command to YAML, recursively including subcommands if present
fn write_command_yaml(output: &mut String, cmd: &CommandDoc, indent: usize) {
    let name = cmd.path.last().unwrap();
    let spaces = " ".repeat(indent);

    if cmd.subcommands.is_empty() {
        // Leaf command
        output.push_str(&format!("{spaces}- \"{name}\"\n"));
    } else {
        // Command with subcommands - use nested format
        output.push_str(&format!("{spaces}- name: \"{name}\"\n"));
        output.push_str(&format!("{spaces}  children:\n"));
        for sub in &cmd.subcommands {
            write_command_yaml(output, sub, indent + 4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_leaf(name: &str) -> CommandDoc {
        CommandDoc {
            path: vec!["stencila".to_string(), name.to_string()],
            description: None,
            long_description: None,
            usage: String::new(),
            examples: None,
            sections: vec![],
            arguments: vec![],
            options: vec![],
            subcommands: vec![],
        }
    }

    fn make_with_subs(name: &str, subs: Vec<CommandDoc>) -> CommandDoc {
        CommandDoc {
            path: vec!["stencila".to_string(), name.to_string()],
            description: None,
            long_description: None,
            usage: String::new(),
            examples: None,
            sections: vec![],
            arguments: vec![],
            options: vec![],
            subcommands: subs,
        }
    }

    #[test]
    fn test_generate_nav_yaml() {
        // Create a minimal root command with some subcommands
        let root = CommandDoc {
            path: vec!["stencila".to_string()],
            description: None,
            long_description: None,
            usage: String::new(),
            examples: None,
            sections: vec![],
            arguments: vec![],
            options: vec![],
            subcommands: vec![
                make_leaf("new"),
                make_leaf("init"),
                make_with_subs(
                    "config",
                    vec![make_leaf("check"), make_leaf("get"), make_leaf("set")],
                ),
            ],
        };

        let yaml = generate_nav_yaml(&root);

        // Check header comments
        assert!(yaml.contains("# Auto-generated navigation"));
        assert!(yaml.contains("items:"));

        // Check first group
        assert!(yaml.contains("label: \"Create & Setup\""));
        assert!(yaml.contains("- \"new\""));
        assert!(yaml.contains("- \"init\""));

        // Check nested config command
        assert!(yaml.contains("- name: \"config\""));
        assert!(yaml.contains("children:"));
        assert!(yaml.contains("- \"check\""));
        assert!(yaml.contains("- \"get\""));
        assert!(yaml.contains("- \"set\""));
    }
}
