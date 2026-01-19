//! Format command documentation as schema nodes

use std::collections::HashMap;

use stencila_schema::{
    Article, ArticleOptions, Block, Inline, Node, TableCell, TableRow,
    shortcuts::{cb, ci, h1, lnk, p, stg, t, tbl, td, th, tr},
};

use crate::extract::{ArgDoc, CommandDoc, HelpSection, PossibleValue};

/// Generate an Article node for a command's documentation
pub fn generate_article(doc: &CommandDoc) -> Node {
    let command_path = doc.path.join(" ");
    // Title with backticks for YAML frontmatter
    let title = format!("\"`{command_path}`\"");

    // Title inlines use CodeInline so it renders with backticks in markdown
    let title_inlines = Some(vec![ci(&command_path)]);

    let description = doc.description.clone();

    // Build frontmatter - note: we wrap in quotes to preserve backticks
    let frontmatter = Some(format!(
        "title: {title}\ndescription: {}",
        description.clone().unwrap_or_default()
    ));

    let content = generate_content(doc);

    Node::Article(Article {
        title: title_inlines,
        frontmatter,
        content,
        options: Box::new(ArticleOptions {
            description,
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Generate the content blocks for a command
fn generate_content(doc: &CommandDoc) -> Vec<Block> {
    let mut content = Vec::new();

    // Description
    if let Some(desc) = &doc.long_description {
        content.push(p([t(desc)]));
    } else if let Some(desc) = &doc.description {
        content.push(p([t(desc)]));
    }

    // Usage section - prefix with full command path
    content.push(h1([t("Usage")]));
    let command_path = doc.path.join(" ");
    let usage = if let Some(rest) = doc.usage.split_once(' ').map(|(_, r)| r) {
        format!("{command_path} {rest}")
    } else {
        command_path.clone()
    };
    content.push(cb(&usage, Some("sh")));

    // Examples section (formatted as bash code block)
    if let Some(examples) = &doc.examples {
        let formatted = format_examples(examples);
        if !formatted.is_empty() {
            content.push(h1([t("Examples")]));
            content.push(cb(formatted, Some("bash")));
        }
    }

    // Subcommands section
    if !doc.subcommands.is_empty() {
        content.push(h1([t("Subcommands")]));

        let mut rows: Vec<TableRow> = vec![tr([th([t("Command")]), th([t("Description")])])];

        for sub in &doc.subcommands {
            let Some(name) = sub.path.last() else {
                continue;
            };
            let desc = sub.description.as_deref().unwrap_or("");
            // Link to the subcommand's page
            let link = if sub.has_subcommands() {
                format!("{name}/index.md")
            } else {
                format!("{name}.md")
            };
            rows.push(tr([td([lnk([ci(name)], link)]), td([t(desc)])]));
        }

        content.push(tbl(rows));
    }

    // Arguments section (as table)
    if !doc.arguments.is_empty() {
        content.push(h1([t("Arguments")]));
        content.extend(arguments_table(&doc.arguments));
    }

    // Options section (as table)
    if !doc.options.is_empty() {
        content.push(h1([t("Options")]));
        content.extend(options_table(&doc.options));
    }

    // Additional sections (Note, Setup Process, Troubleshooting, etc.)
    for section in &doc.sections {
        content.extend(render_help_section(section));
    }

    content
}

/// Render a help section as content blocks
fn render_help_section(section: &HelpSection) -> Vec<Block> {
    let mut blocks = Vec::new();

    blocks.push(h1([t(&section.heading)]));

    // Parse the content into paragraphs (split on blank lines)
    for para in section.content.split("\n\n") {
        let trimmed = para.trim();
        if !trimmed.is_empty() {
            blocks.push(p([t(trimmed)]));
        }
    }

    blocks
}

/// Format examples from after_help/after_long_help
fn format_examples(raw: &str) -> String {
    let lines: Vec<&str> = raw.lines().collect();

    // Skip the "Examples" header if present
    let start = lines
        .iter()
        .position(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed != "Examples"
        })
        .unwrap_or(0);

    lines[start..]
        .iter()
        .map(|line| {
            // Remove leading indentation (typically 2 spaces)
            line.strip_prefix("  ").unwrap_or(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Generate a table of arguments, with separate tables for long possible values
fn arguments_table(args: &[ArgDoc]) -> Vec<Block> {
    let mut rows: Vec<TableRow> = vec![tr([th([t("Name")]), th([t("Description")])])];

    // Collect args with long possible values for separate tables
    // Key: serialized possible values, Value: (display name, possible values reference)
    let mut separate_pv: Vec<(String, &[PossibleValue])> = Vec::new();

    for arg in args {
        let name_display = if arg.is_required {
            format!("<{}>", arg.name.to_uppercase())
        } else {
            format!("[{}]", arg.name.to_uppercase())
        };

        let desc = ensure_full_stop(arg.description.as_deref().unwrap_or(""));
        let mut desc_inlines: Vec<Inline> = vec![t(&desc)];

        // Check if possible values are too long for inline rendering
        let pv_too_long =
            !arg.possible_values.is_empty() && possible_values_text_len(&arg.possible_values) > 100;

        if pv_too_long {
            // Collect for separate table
            separate_pv.push((name_display.clone(), &arg.possible_values));
        } else if !arg.possible_values.is_empty() {
            // Add possible values inline
            desc_inlines.push(t(" Possible values: "));
            for (i, pv) in arg.possible_values.iter().enumerate() {
                if i > 0 {
                    desc_inlines.push(t(", "));
                }
                desc_inlines.push(ci(&pv.name));
                if let Some(pv_desc) = &pv.description {
                    desc_inlines.push(t(format!(" ({pv_desc})")));
                }
            }
            desc_inlines.push(t("."));
        }

        // Add default value if any
        if let Some(default) = &arg.default_value {
            desc_inlines.push(t(" Default value: "));
            desc_inlines.push(ci(default));
            desc_inlines.push(t("."));
        }

        rows.push(tr([
            td([ci(name_display)]),
            TableCell {
                content: vec![Block::Paragraph(stencila_schema::Paragraph::new(
                    desc_inlines,
                ))],
                ..Default::default()
            },
        ]));
    }

    let mut blocks = vec![tbl(rows)];

    // Deduplicate and generate separate possible values tables
    if !separate_pv.is_empty() {
        // Group by possible values content
        let mut grouped: HashMap<Vec<(String, Option<String>)>, Vec<String>> = HashMap::new();
        for (name, pvs) in separate_pv {
            let key: Vec<_> = pvs
                .iter()
                .map(|pv| (pv.name.clone(), pv.description.clone()))
                .collect();
            grouped.entry(key).or_default().push(name);
        }

        // Generate a table for each unique set of possible values
        for (pv_key, names) in grouped {
            let pvs: Vec<PossibleValue> = pv_key
                .into_iter()
                .map(|(name, description)| PossibleValue { name, description })
                .collect();
            let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            blocks.extend(possible_values_table(&name_refs, &pvs));
        }
    }

    blocks
}

/// Generate a table of options, with separate tables for long possible values
fn options_table(opts: &[ArgDoc]) -> Vec<Block> {
    let mut rows: Vec<TableRow> = vec![tr([th([t("Name")]), th([t("Description")])])];

    // Collect options with long possible values for separate tables
    // Uses the --long-name format for display
    let mut separate_pv: Vec<(String, &[PossibleValue])> = Vec::new();

    for opt in opts {
        // Build the flag display (e.g., "-h, --help" or "--config <PATH>")
        let mut flag_parts = Vec::new();
        if let Some(short) = opt.short {
            flag_parts.push(format!("-{short}"));
        }
        if let Some(long) = &opt.long {
            flag_parts.push(format!("--{long}"));
        }

        let flag_display = if flag_parts.is_empty() {
            format!("--{}", opt.name)
        } else {
            flag_parts.join(", ")
        };

        // Add value placeholder if this option takes a value
        let has_value = opt.default_value.is_some() || !opt.possible_values.is_empty();
        let full_display = if has_value {
            format!("{flag_display} <{}>", opt.name.to_uppercase())
        } else {
            flag_display.clone()
        };

        let desc = ensure_full_stop(opt.description.as_deref().unwrap_or(""));
        let mut desc_inlines: Vec<Inline> = vec![t(&desc)];

        // Check if possible values are too long for inline rendering
        let pv_too_long =
            !opt.possible_values.is_empty() && possible_values_text_len(&opt.possible_values) > 100;

        if pv_too_long {
            // Use --long-name format for the separate table header
            let pv_name = opt
                .long
                .as_ref()
                .map(|l| format!("--{l}"))
                .unwrap_or_else(|| format!("--{}", opt.name));
            separate_pv.push((pv_name, &opt.possible_values));
        } else if !opt.possible_values.is_empty() {
            // Add possible values inline
            desc_inlines.push(t(" Possible values: "));
            for (i, pv) in opt.possible_values.iter().enumerate() {
                if i > 0 {
                    desc_inlines.push(t(", "));
                }
                desc_inlines.push(ci(&pv.name));
                if let Some(pv_desc) = &pv.description {
                    desc_inlines.push(t(format!(" ({pv_desc})")));
                }
            }
            desc_inlines.push(t("."));
        }

        // Add default value if any
        if let Some(default) = &opt.default_value {
            desc_inlines.push(t(" Default value: "));
            desc_inlines.push(ci(default));
            desc_inlines.push(t("."));
        }

        rows.push(tr([
            td([ci(full_display)]),
            TableCell {
                content: vec![Block::Paragraph(stencila_schema::Paragraph::new(
                    desc_inlines,
                ))],
                ..Default::default()
            },
        ]));
    }

    let mut blocks = vec![tbl(rows)];

    // Deduplicate and generate separate possible values tables
    if !separate_pv.is_empty() {
        // Group by possible values content
        let mut grouped: HashMap<Vec<(String, Option<String>)>, Vec<String>> = HashMap::new();
        for (name, pvs) in separate_pv {
            let key: Vec<_> = pvs
                .iter()
                .map(|pv| (pv.name.clone(), pv.description.clone()))
                .collect();
            grouped.entry(key).or_default().push(name);
        }

        // Generate a table for each unique set of possible values
        for (pv_key, names) in grouped {
            let pvs: Vec<PossibleValue> = pv_key
                .into_iter()
                .map(|(name, description)| PossibleValue { name, description })
                .collect();
            let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            blocks.extend(possible_values_table(&name_refs, &pvs));
        }
    }

    blocks
}

/// Ensure a string ends with a full stop
fn ensure_full_stop(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    }
}

/// Calculate the approximate text length of possible values when rendered inline
fn possible_values_text_len(possible_values: &[PossibleValue]) -> usize {
    // " Possible values: " prefix
    let base = " Possible values: ".len();
    let values_len: usize = possible_values
        .iter()
        .enumerate()
        .map(|(i, pv)| {
            let sep = if i > 0 { ", ".len() } else { 0 };
            // +2 for backticks around value name in rendered output
            let name_len = pv.name.len() + 2;
            let desc_len = pv
                .description
                .as_ref()
                .map(|d| d.len() + 3) // " (desc)"
                .unwrap_or(0);
            sep + name_len + desc_len
        })
        .sum();
    base + values_len + 1 // +1 for final "."
}

/// Generate a separate table for possible values
///
/// Returns a paragraph header like "**Possible values of `name1`, `name2`**"
/// followed by a two-column table (Value, Description)
fn possible_values_table(names: &[&str], possible_values: &[PossibleValue]) -> Vec<Block> {
    // Build header: **Possible values of `name1`, `name2`**
    let mut header_inlines: Vec<Inline> = vec![t("Possible values of ")];
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            header_inlines.push(t(", "));
        }
        header_inlines.push(ci(name));
    }
    let header = p([stg(header_inlines)]);

    // Build table rows
    let mut rows: Vec<TableRow> = vec![tr([th([t("Value")]), th([t("Description")])])];
    for pv in possible_values {
        let desc = pv.description.as_deref().unwrap_or("");
        rows.push(tr([td([ci(&pv.name)]), td([t(desc)])]));
    }

    vec![header, tbl(rows)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_examples() {
        let raw = "Examples\n  # Create a new article\n  stencila new article";
        let formatted = format_examples(raw);
        assert_eq!(formatted, "# Create a new article\nstencila new article");
    }

    #[test]
    fn test_format_examples_no_header() {
        let raw = "  # Create a new article\n  stencila new article";
        let formatted = format_examples(raw);
        assert_eq!(formatted, "# Create a new article\nstencila new article");
    }

    #[test]
    fn test_ensure_full_stop() {
        assert_eq!(ensure_full_stop("Hello"), "Hello.");
        assert_eq!(ensure_full_stop("Hello."), "Hello.");
        assert_eq!(ensure_full_stop("Hello!"), "Hello!");
        assert_eq!(ensure_full_stop("Hello?"), "Hello?");
        assert_eq!(ensure_full_stop(""), "");
        assert_eq!(ensure_full_stop("  "), "");
    }
}
