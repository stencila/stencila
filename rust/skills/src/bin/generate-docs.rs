use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use eyre::Result;
use serde::Deserialize;
use stencila_skills::SkillInstance;

#[tokio::main]
async fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.join("../..").canonicalize()?;
    let docs_root = repo_root.join("site/docs/skills/builtin");

    fs::create_dir_all(&docs_root)?;

    generate_builtin_skill_pages(&repo_root, &docs_root).await?;

    #[allow(clippy::print_stderr)]
    {
        eprintln!("Generated builtin skills docs at {}", docs_root.display());
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
    skills: Vec<String>,
}

async fn generate_builtin_skill_pages(repo_root: &Path, docs_root: &Path) -> Result<()> {
    let categories_path = repo_root.join(".stencila/skills/categories.yaml");
    let categories_file: CategoriesFile =
        serde_yaml::from_str(&fs::read_to_string(&categories_path)?)?;
    let categories = &categories_file.categories;

    let skills = stencila_skills::discover(repo_root, &[]).await;
    let skills_by_name: BTreeMap<&str, _> = skills.iter().map(|s| (s.name.as_str(), s)).collect();

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

    // Write individual skill pages into category subdirectories
    for category in categories {
        let category_dir = docs_root.join(&category.slug);
        fs::create_dir_all(&category_dir)?;

        for skill_name in &category.skills {
            let skill = skills_by_name.get(skill_name.as_str()).unwrap_or_else(|| {
                panic!(
                    "skill '{}' listed in categories.yaml not found in builtin skills",
                    skill_name
                )
            });
            let raw = fs::read_to_string(skill.path())?;
            let source_path = relative_display(repo_root, skill.path());
            let content = build_skill_page(skill, &raw, &source_path)?;
            fs::write(category_dir.join(format!("{}.md", skill.name)), content)?;
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

        for skill_name in &category.skills {
            let skill = skills_by_name.get(skill_name.as_str()).unwrap_or_else(|| {
                panic!(
                    "skill '{}' listed in categories.yaml not found in builtin skills",
                    skill_name
                )
            });
            writeln!(
                cat_index,
                "- [**{}**](./{}/): {}",
                title_case(&skill.name),
                skill.name,
                skill.description,
            )?;
        }

        fs::write(category_dir.join("index.md"), cat_index)?;
    }

    // Generate top-level index.md
    let mut index = String::new();
    index.push_str(
        "---\n\
title: Builtin Skills\n\
description: Builtin skills bundled with Stencila.\n\
---\n\n\
Builtin skills are bundled with Stencila and loaded as the base skill layer for agent sessions.\n",
    );

    for category in categories {
        writeln!(index, "\n## {}\n", category.title)?;
        writeln!(index, "{}\n", category.prelude)?;

        for skill_name in &category.skills {
            let skill = skills_by_name.get(skill_name.as_str()).unwrap_or_else(|| {
                panic!(
                    "skill '{}' listed in categories.yaml not found in builtin skills",
                    skill_name
                )
            });
            writeln!(
                index,
                "- [**{}**](./{}/{}/) — {}",
                title_case(&skill.name),
                category.slug,
                skill.name,
                skill.description,
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

    // Generate per-category _nav.yaml files to control skill ordering
    for category in categories {
        let mut cat_nav = String::from("items:\n");
        for skill_name in &category.skills {
            writeln!(cat_nav, "  - \"{}\"", skill_name)?;
        }
        fs::write(docs_root.join(&category.slug).join("_nav.yaml"), cat_nav)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Skill page builder
// ---------------------------------------------------------------------------

fn build_skill_page(skill: &SkillInstance, raw: &str, source_path: &str) -> Result<String> {
    let title = title_case(&skill.name);
    let body = extract_body(raw).trim().to_string();

    let mut out = String::new();

    // Site frontmatter — title, description, and keywords
    writeln!(out, "---")?;
    writeln!(out, "title: \"{}\"", title.replace('"', "\\\""))?;
    writeln!(
        out,
        "description: \"{}\"",
        skill.description.replace('"', "\\\"")
    )?;
    if let Some(keywords) = &skill.options.keywords
        && !keywords.is_empty()
    {
        writeln!(out, "keywords:")?;
        for kw in keywords {
            writeln!(out, "  - {kw}")?;
        }
    }
    writeln!(out, "---")?;

    // Description
    writeln!(out, "\n{}\n", skill.description)?;

    // Keywords
    if let Some(keywords) = &skill.options.keywords
        && !keywords.is_empty()
    {
        writeln!(out, "**Keywords:** {}\n", keywords.join(" · "))?;
    }

    // Configuration table
    let mut rows: Vec<(&str, String)> = Vec::new();

    if let Some(tools) = &skill.allowed_tools
        && !tools.is_empty()
    {
        let formatted: Vec<String> = tools.iter().map(|t| format!("`{t}`")).collect();
        rows.push(("Allowed tools", formatted.join(", ")));
    }

    if let Some(compat) = &skill.compatibility {
        rows.push(("Compatibility", compat.clone()));
    }

    if let Some(licenses) = &skill.options.licenses
        && !licenses.is_empty()
    {
        let labels: Vec<String> = licenses.iter().map(license_label).collect();
        rows.push(("License", labels.join(", ")));
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

    // Instructions body
    if !body.is_empty() {
        writeln!(out, "# Instructions\n")?;
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

fn license_label(license: &stencila_schema::CreativeWorkVariantOrString) -> String {
    match license {
        stencila_schema::CreativeWorkVariantOrString::String(value) => value.clone(),
        _ => "(complex)".to_string(),
    }
}
