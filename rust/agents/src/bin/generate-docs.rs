use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use stencila_agents::tool_guard::shell::packs::{Confidence, Pack, all_packs};

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let site_root = PathBuf::from(manifest_dir).join("../../site/docs/agents/tools");

    fs::create_dir_all(site_root.join("shell")).expect("failed to create shell docs dir");

    generate_shell_pack_pages(&site_root);
    replace_between(
        &site_root.join("shell/index.md"),
        "<!-- PACKS_TABLE_START -->",
        "<!-- PACKS_TABLE_END -->",
        &packs_table(),
    );
    replace_between(
        &site_root.join("_nav.yaml"),
        "# SHELL_PAGES_START",
        "# SHELL_PAGES_END",
        &shell_nav_entries(),
    );

    #[allow(clippy::print_stderr)]
    {
        eprintln!("Generated tool guard docs at {}", site_root.display());
    }
}

// ---------------------------------------------------------------------------
// Module mapping
// ---------------------------------------------------------------------------

fn module_for_pack(pack_id: &str) -> &str {
    pack_id.split('.').next().unwrap_or(pack_id)
}

fn module_title(module: &str) -> String {
    match module {
        "hpc" => "HPC".to_string(),
        "ml" => "ML".to_string(),
        "latex" => "LaTeX".to_string(),
        _ => {
            let mut chars = module.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        }
    }
}

fn pack_anchor(name: &str) -> String {
    name.to_lowercase().replace(' ', "-").replace('/', "-")
}

fn packs_by_module() -> BTreeMap<&'static str, Vec<&'static Pack>> {
    let mut map: BTreeMap<&str, Vec<&'static Pack>> = BTreeMap::new();
    for pack in all_packs() {
        let module = module_for_pack(pack.id);
        map.entry(module).or_default().push(pack);
    }
    map
}

// ---------------------------------------------------------------------------
// shell/index.md — packs table (replaces <!-- PACKS_TABLE --> sentinel)
// ---------------------------------------------------------------------------

fn packs_table() -> String {
    let mut out = String::new();
    out.push_str("| Pack ID | Name | Description | Safe Patterns | Destructive Patterns |\n");
    out.push_str("|---------|------|-------------|:-------------:|:--------------------:|\n");
    for pack in all_packs() {
        let module = module_for_pack(pack.id);
        let anchor = pack_anchor(pack.name);
        out.push_str(&format!(
            "| `{}` | [{}]({}#{}) | {} | {} | {} |\n",
            pack.id,
            pack.name,
            module,
            anchor,
            pack.description,
            pack.safe_patterns.len(),
            pack.destructive_patterns.len(),
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// _nav.yaml — shell page entries (replaces # SHELL_PAGES sentinel)
// ---------------------------------------------------------------------------

fn shell_nav_entries() -> String {
    let by_module = packs_by_module();
    let mut out = String::new();
    for module in by_module.keys() {
        out.push_str(&format!("      - \"{module}\"\n"));
    }
    // Remove trailing newline so the sentinel replacement is clean
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

// ---------------------------------------------------------------------------
// shell/<module>.md — fully generated pack pages
// ---------------------------------------------------------------------------

fn generate_shell_pack_pages(root: &Path) {
    let by_module = packs_by_module();

    for (module, packs) in &by_module {
        let title = module_title(module);
        let descriptions: Vec<&str> = packs.iter().map(|p| p.description).collect();
        let description = descriptions.join(". ");

        let mut body = String::new();

        for pack in packs {
            body.push_str(&format!("\n## {}\n\n", pack.name));
            body.push_str(&format!(
                "**Pack ID:** `{}`\n\n{}\n",
                pack.id, pack.description
            ));

            if !pack.safe_patterns.is_empty() {
                body.push_str("\n### Safe patterns\n\n");
                body.push_str("| Rule ID | Pattern |\n");
                body.push_str("|---------|--------|\n");
                for rule in pack.safe_patterns {
                    body.push_str(&format!(
                        "| `{}.{}` | `{}` |\n",
                        pack.id,
                        rule.id,
                        escape_markdown_table(rule.pattern),
                    ));
                }
            }

            if !pack.destructive_patterns.is_empty() {
                body.push_str("\n### Destructive patterns\n\n");
                body.push_str("| Rule ID | Reason | Suggestion | Confidence |\n");
                body.push_str("|---------|--------|------------|:----------:|\n");
                for rule in pack.destructive_patterns {
                    let confidence = match rule.confidence {
                        Confidence::High => "High",
                        Confidence::Medium => "Medium",
                    };
                    body.push_str(&format!(
                        "| `{}.{}` | {} | {} | {} |\n",
                        pack.id,
                        rule.id,
                        escape_markdown_table(rule.reason),
                        escape_markdown_table(rule.suggestion),
                        confidence,
                    ));
                }
            }
        }

        let pack_names: Vec<&str> = packs.iter().map(|p| p.name).collect();
        let preface = if pack_names.len() == 1 {
            format!(
                "This page lists the safe and destructive patterns in the **{}** shell guard pack. \
                 See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.\n",
                pack_names[0],
            )
        } else {
            format!(
                "This page lists the safe and destructive patterns in the {} shell guard packs. \
                 See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.\n",
                join_names(&pack_names),
            )
        };

        let source_path = format!("rust/agents/src/tool_guard/shell/packs/{module}.rs");
        let content = format!(
            "---\ntitle: \"{title}\"\ndescription: \"{description}\"\n---\n\n{preface}{body}\n\
             ---\n\n\
             This documentation was generated from [`{source_path}`](https://github.com/stencila/stencila/blob/main/{source_path}).\n"
        );
        let path = root.join(format!("shell/{module}.md"));
        fs::write(&path, content)
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    }
}

// ---------------------------------------------------------------------------
// Sentinel replacement
// ---------------------------------------------------------------------------

/// Replace everything between `start` and `end` sentinel lines (inclusive)
/// with `start\n{replacement}\nend`. This is idempotent — running it again
/// replaces the previously generated content.
fn replace_between(path: &Path, start: &str, end: &str, replacement: &str) {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    let start_idx = content
        .find(start)
        .unwrap_or_else(|| panic!("start sentinel {start:?} not found in {}", path.display()));
    let end_idx = content[start_idx..]
        .find(end)
        .unwrap_or_else(|| panic!("end sentinel {end:?} not found in {}", path.display()));

    // Find the end of the end-sentinel line
    let abs_end = start_idx + end_idx;
    let line_end = content[abs_end..]
        .find('\n')
        .map(|i| abs_end + i + 1)
        .unwrap_or(content.len());

    let mut new_content = String::with_capacity(content.len());
    new_content.push_str(&content[..start_idx]);
    new_content.push_str(start);
    new_content.push('\n');
    new_content.push_str(replacement);
    if !replacement.ends_with('\n') {
        new_content.push('\n');
    }

    // Preserve the indentation of the end sentinel
    let end_line_start = content[..abs_end].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let indent = &content[end_line_start..abs_end];
    // Only add indent if it's all whitespace (i.e., the end sentinel was indented)
    if indent.chars().all(char::is_whitespace) {
        new_content.push_str(indent);
    }
    new_content.push_str(end);
    new_content.push('\n');
    new_content.push_str(&content[line_end..]);

    fs::write(path, new_content)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
}

fn join_names(names: &[&str]) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].to_string(),
        2 => format!("**{}** and **{}**", names[0], names[1]),
        _ => {
            let (last, rest) = names.split_last().unwrap();
            let parts: Vec<String> = rest.iter().map(|n| format!("**{n}**")).collect();
            format!("{}, and **{last}**", parts.join(", "))
        }
    }
}

fn escape_markdown_table(s: &str) -> String {
    s.replace('|', "\\|")
}
