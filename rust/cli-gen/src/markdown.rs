//! Format command documentation as schema nodes

use stencila_schema::{
    Article, ArticleOptions, Block, Inline, Node, TableCell, TableRow,
    shortcuts::{cb, ci, h1, lnk, p, t, tbl, td, th, tr},
};

use crate::extract::{ArgDoc, CommandDoc, HelpSection};

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
        content.push(arguments_table(&doc.arguments));
    }

    // Options section (as table)
    if !doc.options.is_empty() {
        content.push(h1([t("Options")]));
        content.push(options_table(&doc.options));
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

/// Generate a table of arguments
fn arguments_table(args: &[ArgDoc]) -> Block {
    let mut rows: Vec<TableRow> = vec![tr([th([t("Name")]), th([t("Description")])])];

    for arg in args {
        let name_display = if arg.is_required {
            format!("<{}>", arg.name.to_uppercase())
        } else {
            format!("[{}]", arg.name.to_uppercase())
        };

        let desc = ensure_full_stop(arg.description.as_deref().unwrap_or(""));
        let mut desc_inlines: Vec<Inline> = vec![t(&desc)];

        // Add possible values if any
        if !arg.possible_values.is_empty() {
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

    tbl(rows)
}

/// Generate a table of options
fn options_table(opts: &[ArgDoc]) -> Block {
    let mut rows: Vec<TableRow> = vec![tr([th([t("Name")]), th([t("Description")])])];

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
            flag_display
        };

        let desc = ensure_full_stop(opt.description.as_deref().unwrap_or(""));
        let mut desc_inlines: Vec<Inline> = vec![t(&desc)];

        // Add possible values if any
        if !opt.possible_values.is_empty() {
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

    tbl(rows)
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
