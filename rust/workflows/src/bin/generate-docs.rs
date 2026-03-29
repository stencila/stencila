use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use eyre::Result;
use serde::Deserialize;
use stencila_workflows::WorkflowInstance;

#[tokio::main]
async fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.join("../..").canonicalize()?;
    let docs_root = repo_root.join("site/docs/workflows/builtin");

    fs::create_dir_all(&docs_root)?;

    generate_builtin_workflow_pages(&repo_root, &docs_root).await?;

    #[allow(clippy::print_stderr)]
    {
        eprintln!(
            "Generated builtin workflows docs at {}",
            docs_root.display()
        );
    }

    Ok(())
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
    workflows: Vec<String>,
}

#[derive(Deserialize)]
struct AgentCategoriesFile {
    categories: Vec<AgentCategory>,
}

#[derive(Deserialize)]
struct AgentCategory {
    slug: String,
    agents: Vec<String>,
}

/// Build a map from agent name to its category slug by reading the agents
/// categories.yaml file. Returns an empty map if the file is missing.
fn load_agent_to_category(repo_root: &Path) -> BTreeMap<String, String> {
    let path = repo_root.join(".stencila/agents/categories.yaml");
    let Ok(content) = fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    let Ok(file) = serde_yaml::from_str::<AgentCategoriesFile>(&content) else {
        return BTreeMap::new();
    };
    let mut map = BTreeMap::new();
    for cat in file.categories {
        for agent in cat.agents {
            map.insert(agent, cat.slug.clone());
        }
    }
    map
}

async fn generate_builtin_workflow_pages(repo_root: &Path, docs_root: &Path) -> Result<()> {
    let categories_path = repo_root.join(".stencila/workflows/categories.yaml");
    let categories_file: CategoriesFile =
        serde_yaml::from_str(&fs::read_to_string(&categories_path)?)?;
    let categories = &categories_file.categories;

    let workflows = stencila_workflows::list_builtin().await;
    let workflows_by_name: BTreeMap<&str, _> =
        workflows.iter().map(|w| (w.name.as_str(), w)).collect();

    let agent_to_category = load_agent_to_category(repo_root);

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

    // Write individual workflow pages into category subdirectories
    for category in categories {
        let category_dir = docs_root.join(&category.slug);
        fs::create_dir_all(&category_dir)?;

        for workflow_name in &category.workflows {
            let workflow = workflows_by_name
                .get(workflow_name.as_str())
                .unwrap_or_else(|| {
                    panic!(
                        "workflow '{}' listed in categories.yaml not found in builtin workflows",
                        workflow_name
                    )
                });
            let raw = fs::read_to_string(workflow.path())?;
            let source_path = relative_display(repo_root, workflow.path());
            let content = build_workflow_page(workflow, &raw, &source_path, &agent_to_category)?;
            fs::write(category_dir.join(format!("{}.md", workflow.name)), content)?;
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

        for workflow_name in &category.workflows {
            let workflow = workflows_by_name
                .get(workflow_name.as_str())
                .unwrap_or_else(|| {
                    panic!(
                        "workflow '{}' listed in categories.yaml not found in builtin workflows",
                        workflow_name
                    )
                });
            writeln!(
                cat_index,
                "- [**{}**](./{}/): {}",
                title_case(&workflow.name),
                workflow.name,
                workflow.description,
            )?;
        }

        fs::write(category_dir.join("index.md"), cat_index)?;
    }

    // Generate top-level index.md
    let mut index = String::new();
    index.push_str(
        "---\n\
title: Builtin Workflows\n\
description: Builtin workflows bundled with Stencila.\n\
---\n\n\
Builtin workflows are bundled with Stencila and can be listed and run without adding local workflow files.\n",
    );

    for category in categories {
        writeln!(index, "\n## {}\n", category.title)?;
        writeln!(index, "{}\n", category.prelude)?;

        for workflow_name in &category.workflows {
            let workflow = workflows_by_name
                .get(workflow_name.as_str())
                .unwrap_or_else(|| {
                    panic!(
                        "workflow '{}' listed in categories.yaml not found in builtin workflows",
                        workflow_name
                    )
                });
            writeln!(
                index,
                "- [**{}**](./{}/{}/) — {}",
                title_case(&workflow.name),
                category.slug,
                workflow.name,
                workflow.description,
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

    // Generate per-category _nav.yaml files to control workflow ordering
    for category in categories {
        let mut cat_nav = String::from("items:\n");
        for workflow_name in &category.workflows {
            writeln!(cat_nav, "  - \"{}\"", workflow_name)?;
        }
        fs::write(docs_root.join(&category.slug).join("_nav.yaml"), cat_nav)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Workflow page builder
// ---------------------------------------------------------------------------

fn build_workflow_page(
    workflow: &WorkflowInstance,
    raw: &str,
    source_path: &str,
    agent_to_category: &BTreeMap<String, String>,
) -> Result<String> {
    let title = title_case(&workflow.name);
    let body = extract_body(raw).trim().to_string();

    let mut out = String::new();

    // Site frontmatter — title, description, and keywords
    writeln!(out, "---")?;
    writeln!(out, "title: \"{}\"", title.replace('"', "\\\""))?;
    writeln!(
        out,
        "description: \"{}\"",
        workflow.description.replace('"', "\\\"")
    )?;
    if let Some(keywords) = &workflow.options.keywords
        && !keywords.is_empty()
    {
        writeln!(out, "keywords:")?;
        for kw in keywords {
            writeln!(out, "  - {kw}")?;
        }
    }
    writeln!(out, "---")?;

    // Description
    writeln!(out, "\n{}\n", workflow.description)?;

    // Keywords
    if let Some(keywords) = &workflow.options.keywords
        && !keywords.is_empty()
    {
        writeln!(out, "**Keywords:** {}\n", keywords.join(" · "))?;
    }

    // When to use
    if let Some(items) = &workflow.when_to_use
        && !items.is_empty()
    {
        writeln!(out, "# When to use\n")?;
        for item in items {
            writeln!(out, "- {item}")?;
        }
        writeln!(out)?;
    }

    // When not to use
    if let Some(items) = &workflow.when_not_to_use
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

    if let Some(goal) = workflow.goal_hint.as_ref().or(workflow.goal.as_ref()) {
        rows.push(("Goal", goal.clone()));
    }

    let agents = workflow.agent_references();
    if !agents.is_empty() {
        let formatted: Vec<String> = agents
            .iter()
            .map(|a| {
                if let Some(slug) = agent_to_category.get(a.as_str()) {
                    format!("[`{a}`](/docs/agents/builtin/{slug}/{a}/)")
                } else {
                    format!("`{a}`")
                }
            })
            .collect();
        rows.push(("Referenced agents", formatted.join(", ")));
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

    // Pipeline / body
    if !body.is_empty() {
        writeln!(out, "# Pipeline\n")?;
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn escape_markdown_table(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}
