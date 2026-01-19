//! Extract command documentation from clap's Command tree

use clap::{Arg, Command};

/// Represents a CLI command and its documentation
#[derive(Debug)]
pub struct CommandDoc {
    /// Full command path e.g. ["stencila", "site", "push"]
    pub path: Vec<String>,
    /// Short description (from clap's `about`)
    pub description: Option<String>,
    /// Long description (from clap's `long_about`)
    pub long_description: Option<String>,
    /// Usage string
    pub usage: String,
    /// Examples (from `after_long_help` or `after_help`, under "Examples" heading)
    pub examples: Option<String>,
    /// Additional help sections (e.g., "Note", "Setup Process", "Troubleshooting")
    pub sections: Vec<HelpSection>,
    /// Positional arguments
    pub arguments: Vec<ArgDoc>,
    /// Optional arguments/flags
    pub options: Vec<ArgDoc>,
    /// Child commands (recursive)
    pub subcommands: Vec<CommandDoc>,
}

/// A section in the after_long_help content
#[derive(Debug)]
pub struct HelpSection {
    /// Section heading (e.g., "Note", "Setup Process")
    pub heading: String,
    /// Section content
    pub content: String,
}

impl CommandDoc {
    /// Whether this command has subcommands
    pub fn has_subcommands(&self) -> bool {
        !self.subcommands.is_empty()
    }
}

/// Represents a CLI argument or option
#[derive(Debug)]
pub struct ArgDoc {
    /// Argument name
    pub name: String,
    /// Short flag (e.g., 'h' for -h)
    pub short: Option<char>,
    /// Long flag (e.g., "help" for --help)
    pub long: Option<String>,
    /// Description of the argument
    pub description: Option<String>,
    /// Default value if any
    pub default_value: Option<String>,
    /// Possible values for the argument
    pub possible_values: Vec<PossibleValue>,
    /// Whether the argument is required
    pub is_required: bool,
}

/// A possible value for an argument with optional description
#[derive(Debug)]
pub struct PossibleValue {
    pub name: String,
    pub description: Option<String>,
}

/// Extract documentation from a clap Command recursively
pub fn extract_command_doc(cmd: &Command, path: Vec<String>) -> CommandDoc {
    // Extract subcommands recursively
    let subcommands: Vec<CommandDoc> = cmd
        .get_subcommands()
        .filter(|sub| !sub.is_hide_set())
        .map(|sub| {
            let mut child_path = path.clone();
            child_path.push(get_canonical_name(sub));
            extract_command_doc(sub, child_path)
        })
        .collect();

    // Build usage string
    let usage = cmd
        .clone()
        .render_usage()
        .to_string()
        .replace("Usage: ", "");

    // Parse after_long_help into examples and other sections
    let after_help = cmd
        .get_after_long_help()
        .or(cmd.get_after_help())
        .map(|s| strip_ansi_codes(&s.to_string()));

    let (examples, sections) = parse_help_sections(after_help.as_deref());

    CommandDoc {
        path,
        description: cmd.get_about().map(|s| strip_ansi_codes(&s.to_string())),
        long_description: cmd
            .get_long_about()
            .map(|s| strip_ansi_codes(&s.to_string())),
        usage,
        examples,
        sections,
        arguments: extract_arguments(cmd),
        options: extract_options(cmd),
        subcommands,
    }
}

/// Get the canonical name for a command (prefer primary name over aliases)
fn get_canonical_name(cmd: &Command) -> String {
    cmd.get_name().to_string()
}

/// Parse after_long_help content into examples and other sections
///
/// Sections are delimited by headings in the format "Heading" (after ANSI stripping).
/// The "Examples" section is returned separately as it's formatted as code.
fn parse_help_sections(content: Option<&str>) -> (Option<String>, Vec<HelpSection>) {
    let Some(content) = content else {
        return (None, Vec::new());
    };

    let mut examples = None;
    let mut sections = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_content = String::new();

    for line in content.lines() {
        // Check if this line is a heading (text that was wrapped in <bold><b>...</b></bold>)
        // After ANSI stripping, these appear as plain text lines that are section headers
        let trimmed = line.trim();

        // Detect section headings - they're typically short, capitalized words/phrases
        // that appear alone on a line and match known patterns
        if is_section_heading(trimmed) {
            // Save previous section if any
            if let Some(heading) = current_heading.take() {
                let content_trimmed = dedent_content(&current_content);
                if heading.eq_ignore_ascii_case("examples") {
                    examples = Some(content_trimmed);
                } else {
                    sections.push(HelpSection {
                        heading,
                        content: content_trimmed,
                    });
                }
            }
            current_heading = Some(trimmed.to_string());
            current_content.clear();
        } else {
            // Add to current content
            if !current_content.is_empty() || !trimmed.is_empty() {
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
            }
        }
    }

    // Save final section
    if let Some(heading) = current_heading {
        let content_trimmed = dedent_content(&current_content);
        if heading.eq_ignore_ascii_case("examples") {
            examples = Some(content_trimmed);
        } else {
            sections.push(HelpSection {
                heading,
                content: content_trimmed,
            });
        }
    } else if !current_content.is_empty() {
        // Content without a heading - treat as examples for backwards compatibility
        examples = Some(dedent_content(&current_content));
    }

    (examples, sections)
}

/// Check if a line is a section heading
fn is_section_heading(line: &str) -> bool {
    // Known section headings (case-insensitive)
    let known_headings = [
        "examples",
        "example",
        "note",
        "notes",
        "setup process",
        "troubleshooting",
        "see also",
        "warning",
        "important",
    ];

    let lower = line.to_lowercase();
    known_headings.contains(&lower.as_str())
}

/// Remove common leading indentation from content
fn dedent_content(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Find minimum indentation (excluding empty lines)
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Remove the common indentation
    lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                &line[min_indent..]
            } else {
                line.trim()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Extract positional arguments from a command
fn extract_arguments(cmd: &Command) -> Vec<ArgDoc> {
    cmd.get_arguments()
        .filter(|arg| arg.is_positional() && !arg.is_hide_set())
        .map(extract_arg_doc)
        .collect()
}

/// Extract optional arguments (flags/options) from a command
fn extract_options(cmd: &Command) -> Vec<ArgDoc> {
    cmd.get_arguments()
        .filter(|arg| !arg.is_positional() && !arg.is_hide_set())
        .map(extract_arg_doc)
        .collect()
}

/// Extract documentation from a single argument
fn extract_arg_doc(arg: &Arg) -> ArgDoc {
    let possible_values: Vec<PossibleValue> = arg
        .get_possible_values()
        .into_iter()
        .filter(|pv| !pv.is_hide_set())
        .map(|pv| PossibleValue {
            name: pv.get_name().to_string(),
            description: pv.get_help().map(|s| strip_ansi_codes(&s.to_string())),
        })
        .collect();

    let default_value = arg
        .get_default_values()
        .first()
        .map(|v| v.to_string_lossy().to_string());

    ArgDoc {
        name: arg.get_id().to_string(),
        short: arg.get_short(),
        long: arg.get_long().map(|s| s.to_string()),
        description: arg.get_help().map(|s| strip_ansi_codes(&s.to_string())),
        default_value,
        possible_values,
        is_required: arg.is_required_set(),
    }
}

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    // ANSI escape codes start with ESC[ (0x1B 0x5B) or ESC (0x1B)
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip the escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                // Skip until we hit a letter (end of escape sequence)
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_codes() {
        assert_eq!(strip_ansi_codes("hello"), "hello");
        assert_eq!(strip_ansi_codes("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(
            strip_ansi_codes("\x1b[1;32mbold green\x1b[0m"),
            "bold green"
        );
    }
}
