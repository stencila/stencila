use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use eyre::Result;
use serde::Deserialize;
use stencila_agents::definition::AgentInstance;
use stencila_agents::tool_guard::shell::packs::{Confidence, Pack, all_packs};

#[tokio::main]
async fn main() -> Result<()> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_root = PathBuf::from(manifest_dir).join("../..").canonicalize()?;
    let site_root = repo_root.join("site/docs/agents/tools");
    let builtin_root = repo_root.join("site/docs/agents/builtin");

    fs::create_dir_all(site_root.join("shell")).expect("failed to create shell docs dir");
    fs::create_dir_all(&builtin_root)?;

    generate_shell_pack_pages(&site_root);
    generate_builtin_agent_pages(&repo_root, &builtin_root).await?;
    replace_between(
        &site_root.join("shell/index.md"),
        "<!-- PACKS_TABLE_START -->",
        "<!-- PACKS_TABLE_END -->",
        &packs_table(),
    );

    #[allow(clippy::print_stderr)]
    {
        eprintln!("Generated tool guard docs at {}", site_root.display());
        eprintln!(
            "Generated builtin agents docs at {}",
            builtin_root.display()
        );
    }

    Ok(())
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
    name.to_lowercase().replace([' ', '/'], "-")
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
// categories.yaml — canonical category definitions
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CategoriesFile {
    categories: Vec<Category>,
}

#[derive(Deserialize)]
struct Category {
    slug: String,
    title: String,
    prelude: String,
    agents: Vec<String>,
}

#[derive(Deserialize)]
struct SkillCategoriesFile {
    categories: Vec<SkillCategory>,
}

#[derive(Deserialize)]
struct SkillCategory {
    slug: String,
    skills: Vec<String>,
}

/// Build a map from skill name to its category slug by reading the skills
/// categories.yaml file. Returns an empty map if the file is missing.
fn load_skill_to_category(repo_root: &Path) -> BTreeMap<String, String> {
    let path = repo_root.join(".stencila/skills/categories.yaml");
    let Ok(content) = fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    let Ok(file) = serde_yaml::from_str::<SkillCategoriesFile>(&content) else {
        return BTreeMap::new();
    };
    let mut map = BTreeMap::new();
    for cat in file.categories {
        for skill in cat.skills {
            map.insert(skill, cat.slug.clone());
        }
    }
    map
}

async fn generate_builtin_agent_pages(repo_root: &Path, docs_root: &Path) -> Result<()> {
    let categories_path = repo_root.join(".stencila/agents/categories.yaml");
    let categories_file: CategoriesFile =
        serde_yaml::from_str(&fs::read_to_string(&categories_path)?)?;
    let categories = &categories_file.categories;

    let agents = stencila_agents::definition::list_builtin().await;
    let agents_by_name: BTreeMap<&str, _> = agents.iter().map(|a| (a.name.as_str(), a)).collect();

    let skill_to_category = load_skill_to_category(repo_root);

    // Clean the docs directory (remove old flat files and category subdirs)
    if docs_root.exists() {
        for entry in fs::read_dir(docs_root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else if path.file_name().is_some_and(|name| {
                let name = name.to_string_lossy();
                name != "index.md" && name != "_nav.yaml"
            }) {
                fs::remove_file(&path)?;
            }
        }
    }

    // Write individual agent pages into category subdirectories
    for category in categories {
        let category_dir = docs_root.join(&category.slug);
        fs::create_dir_all(&category_dir)?;

        for agent_name in &category.agents {
            let agent = agents_by_name.get(agent_name.as_str()).unwrap_or_else(|| {
                panic!(
                    "agent '{}' listed in categories.yaml not found in builtin agents",
                    agent_name
                )
            });
            let raw = fs::read_to_string(agent.path())?;
            let source_path = relative_display(repo_root, agent.path());
            let content = build_agent_page(agent, &raw, &source_path, &skill_to_category)?;
            fs::write(category_dir.join(format!("{}.md", agent.name)), content)?;
        }
    }

    // Generate per-category index.md pages
    for category in categories {
        let category_dir = docs_root.join(&category.slug);
        let mut cat_index = String::new();
        writeln!(cat_index, "---")?;
        writeln!(
            cat_index,
            "title: \"{}\"",
            category.title.replace('"', "\\\"")
        )?;
        writeln!(
            cat_index,
            "description: \"{}\"",
            category.prelude.replace('"', "\\\"")
        )?;
        writeln!(cat_index, "---\n")?;
        writeln!(cat_index, "{}\n", category.prelude)?;

        for agent_name in &category.agents {
            let agent = agents_by_name.get(agent_name.as_str()).unwrap_or_else(|| {
                panic!(
                    "agent '{}' listed in categories.yaml not found in builtin agents",
                    agent_name
                )
            });
            writeln!(
                cat_index,
                "- [**{}**](./{}/): {}",
                title_case(&agent.name),
                agent.name,
                agent.description,
            )?;
        }

        fs::write(category_dir.join("index.md"), cat_index)?;
    }

    // Generate top-level index.md
    let mut index = String::new();
    index.push_str(
        "---\n\
title: Builtin Agents\n\
description: Builtin agents bundled with Stencila.\n\
---\n\n\
Builtin agents are bundled with Stencila and can be used without creating workspace or user-level agent definitions.\n",
    );

    for category in categories {
        writeln!(index, "\n## {}\n", category.title)?;
        writeln!(index, "{}\n", category.prelude)?;

        for agent_name in &category.agents {
            let agent = agents_by_name.get(agent_name.as_str()).unwrap_or_else(|| {
                panic!(
                    "agent '{}' listed in categories.yaml not found in builtin agents",
                    agent_name
                )
            });
            writeln!(
                index,
                "- [**{}**](./{}/{}/) — {}",
                title_case(&agent.name),
                category.slug,
                agent.name,
                agent.description,
            )?;
        }
    }

    fs::write(docs_root.join("index.md"), index)?;

    // Generate _nav.yaml listing category subdirectories in order
    let mut nav = String::from("items:\n");
    for category in categories {
        writeln!(nav, "  - \"{}\"", category.slug)?;
    }
    fs::write(docs_root.join("_nav.yaml"), nav)?;

    // Generate per-category _nav.yaml files to control agent ordering
    for category in categories {
        let mut cat_nav = String::from("items:\n");
        for agent_name in &category.agents {
            writeln!(cat_nav, "  - \"{}\"", agent_name)?;
        }
        fs::write(docs_root.join(&category.slug).join("_nav.yaml"), cat_nav)?;
    }

    Ok(())
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
            let (last, rest) = names.split_last().expect("length checked to be >= 3");
            let parts: Vec<String> = rest.iter().map(|n| format!("**{n}**")).collect();
            format!("{}, and **{last}**", parts.join(", "))
        }
    }
}

fn escape_markdown_table(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

// ---------------------------------------------------------------------------
// Agent page builder
// ---------------------------------------------------------------------------

fn build_agent_page(
    agent: &AgentInstance,
    raw: &str,
    source_path: &str,
    skill_to_category: &BTreeMap<String, String>,
) -> Result<String> {
    let title = title_case(&agent.name);
    let body = extract_body(raw).trim().to_string();

    let mut out = String::new();

    // Site frontmatter — title, description, and keywords
    writeln!(out, "---")?;
    writeln!(out, "title: \"{}\"", title.replace('"', "\\\""))?;
    writeln!(
        out,
        "description: \"{}\"",
        agent.description.replace('"', "\\\"")
    )?;
    if let Some(keywords) = &agent.options.keywords
        && !keywords.is_empty()
    {
        writeln!(out, "keywords:")?;
        for kw in keywords {
            writeln!(out, "  - {kw}")?;
        }
    }
    writeln!(out, "---")?;

    // Description
    writeln!(out, "\n{}\n", agent.description)?;

    // Keywords
    if let Some(keywords) = &agent.options.keywords
        && !keywords.is_empty()
    {
        writeln!(out, "**Keywords:** {}\n", keywords.join(" · "))?;
    }

    // When to use
    if let Some(items) = &agent.when_to_use
        && !items.is_empty()
    {
        writeln!(out, "# When to use\n")?;
        for item in items {
            writeln!(out, "- {item}")?;
        }
        writeln!(out)?;
    }

    // When not to use
    if let Some(items) = &agent.when_not_to_use
        && !items.is_empty()
    {
        writeln!(out, "# When not to use\n")?;
        for item in items {
            writeln!(out, "- {item}")?;
        }
        writeln!(out)?;
    }

    // Configuration table
    let mut rows: Vec<(&str, String)> = Vec::new();

    let model_preference = agent
        .models
        .as_ref()
        .filter(|v| !v.is_empty())
        .map(|v| {
            v.iter()
                .map(|m| format!("`{m}`"))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .or_else(|| agent.model_size.as_ref().map(|s| format!("`{s}`")));
    if let Some(val) = model_preference {
        rows.push(("Model", val));
    }

    if let Some(val) = &agent.reasoning_effort {
        rows.push(("Reasoning effort", format!("`{val}`")));
    }

    if let Some(val) = &agent.trust_level {
        rows.push(("Trust level", format!("`{val}`")));
    }

    if let Some(tools) = &agent.allowed_tools
        && !tools.is_empty()
    {
        let formatted: Vec<String> = tools.iter().map(|t| format!("`{t}`")).collect();
        rows.push(("Tools", formatted.join(", ")));
    }

    if let Some(skills) = &agent.allowed_skills {
        if skills.is_empty() {
            rows.push(("Skills", "none".to_string()));
        } else {
            let formatted: Vec<String> = skills
                .iter()
                .map(|s| {
                    if let Some(slug) = skill_to_category.get(s.as_str()) {
                        format!("[`{s}`](/docs/skills/builtin/{slug}/{s}/)")
                    } else {
                        format!("`{s}`")
                    }
                })
                .collect();
            rows.push(("Skills", formatted.join(", ")));
        }
    }

    if let Some(true) = agent.options.enable_mcp {
        rows.push(("MCP", "enabled".to_string()));
    } else if let Some(false) = agent.options.enable_mcp {
        rows.push(("MCP", "disabled".to_string()));
    }

    if let Some(true) = agent.options.enable_mcp_codemode {
        rows.push(("MCP codemode", "enabled".to_string()));
    } else if let Some(false) = agent.options.enable_mcp_codemode {
        rows.push(("MCP codemode", "disabled".to_string()));
    }

    if !rows.is_empty() {
        writeln!(out, "# Configuration\n")?;
        writeln!(out, "| Property | Value |")?;
        writeln!(out, "| -------- | ----- |")?;
        for (key, value) in &rows {
            writeln!(out, "| {} | {} |", key, escape_markdown_table(value))?;
        }
        writeln!(out)?;
    }

    // Thematic break before the system prompt body
    if !body.is_empty() {
        writeln!(out, "# Prompt\n")?;
        writeln!(out, "{body}\n")?;
    }

    // Footer
    writeln!(out, "---\n")?;
    writeln!(
        out,
        "This page was generated from [`{source_path}`]\
         (https://github.com/stencila/stencila/blob/main/{source_path})."
    )?;

    Ok(out)
}

/// Extract the Markdown body after YAML frontmatter.
fn extract_body(raw: &str) -> &str {
    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(index) = rest.find("\n---\n")
    {
        return &rest[index + 5..];
    }
    raw
}

fn title_case(name: &str) -> String {
    name.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
