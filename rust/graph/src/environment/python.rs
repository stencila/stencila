//! Python environment manifest parsing.
//!
//! Python environment declarations are split across standards and tools:
//! `pyproject.toml` may use PEP 621 project metadata, Poetry-specific tables,
//! or both, while pip requirements files remain common in smaller projects and
//! generated workflows. This module extracts direct package declarations from
//! those sources without invoking Python packaging tools.
//!
//! The parser keeps the result intentionally shallow. Python requirement
//! strings can contain environment markers, extras, direct references, VCS URLs,
//! local paths, and constraints. The graph only needs a package node and enough
//! preserved detail to explain where the dependency came from; resolver-level
//! interpretation belongs in a later ecosystem-specific pass.

use std::{fs, path::Path};

use eyre::{Result, WrapErr};
use toml::Value;

use super::{Dependency, Ecosystem, EnvironmentManifest};
use crate::ids::WorkspaceRelPath;

/// Parse a `pyproject.toml` manifest.
///
/// This handles the two project metadata layouts most likely to contain direct
/// dependencies: PEP 621 `project.dependencies` and Poetry's `tool.poetry`
/// tables. Supporting both in one manifest pass lets mixed or migrating
/// projects still produce one Python environment node.
pub(super) fn parse_pyproject(path: &Path, rel: &WorkspaceRelPath) -> Result<EnvironmentManifest> {
    let text =
        fs::read_to_string(path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    let value = Value::Table(
        text.parse::<toml::Table>()
            .wrap_err_with(|| format!("unable to parse {}", path.display()))?,
    );

    let mut dependencies = Vec::new();

    if let Some(project) = value.get("project").and_then(Value::as_table) {
        push_requirement_array(
            project.get("dependencies"),
            "project.dependencies",
            &mut dependencies,
        );

        if let Some(optional) = project
            .get("optional-dependencies")
            .and_then(Value::as_table)
        {
            for (group, requirements) in optional {
                push_requirement_array(
                    Some(requirements),
                    format!("project.optional-dependencies.{group}"),
                    &mut dependencies,
                );
            }
        }
    }

    if let Some(poetry) = value
        .get("tool")
        .and_then(Value::as_table)
        .and_then(|tool| tool.get("poetry"))
        .and_then(Value::as_table)
    {
        push_dependency_table(
            poetry.get("dependencies"),
            "tool.poetry.dependencies",
            &mut dependencies,
        );
        push_dependency_table(
            poetry.get("dev-dependencies"),
            "tool.poetry.dev-dependencies",
            &mut dependencies,
        );

        if let Some(groups) = poetry.get("group").and_then(Value::as_table) {
            for (group, spec) in groups {
                let dependencies_table =
                    spec.as_table().and_then(|table| table.get("dependencies"));
                push_dependency_table(
                    dependencies_table,
                    format!("tool.poetry.group.{group}.dependencies"),
                    &mut dependencies,
                );
            }
        }
    }

    Ok(EnvironmentManifest {
        ecosystem: Ecosystem::Python,
        rel: rel.clone(),
        dependencies,
        lockfile_names: vec![
            "uv.lock".to_string(),
            "poetry.lock".to_string(),
            "Pipfile.lock".to_string(),
        ],
    })
}

/// Parse a pip requirements file.
///
/// Requirements files do not normally name a lockfile sibling, so this parser
/// contributes dependencies only. That keeps `requirements.txt` useful for
/// environment discovery without implying that it is a reproducible lockfile.
pub(super) fn parse_requirements(
    path: &Path,
    rel: &WorkspaceRelPath,
) -> Result<EnvironmentManifest> {
    let text =
        fs::read_to_string(path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    let dependencies = text
        .lines()
        .filter_map(|line| parse_requirement_line(line, "requirements"))
        .collect();

    Ok(EnvironmentManifest {
        ecosystem: Ecosystem::Python,
        rel: rel.clone(),
        dependencies,
        lockfile_names: vec![],
    })
}

/// Push dependency strings from a PEP 621 dependency array.
///
/// PEP 621 stores dependencies as requirement strings. Passing each string
/// through the same line parser as requirements files keeps handling of extras,
/// markers, and direct references consistent across Python manifest formats.
fn push_requirement_array(
    value: Option<&Value>,
    group: impl Into<String>,
    dependencies: &mut Vec<Dependency>,
) {
    let group = group.into();
    let Some(requirements) = value.and_then(Value::as_array) else {
        return;
    };

    dependencies.extend(requirements.iter().filter_map(|requirement| {
        requirement
            .as_str()
            .and_then(|line| parse_requirement_line(line, group.clone()))
    }));
}

/// Push Poetry dependency table entries.
///
/// Poetry uses TOML tables rather than requirement strings and includes the
/// Python interpreter constraint in the same table. The interpreter entry is
/// skipped because it describes the runtime, not a package dependency node.
fn push_dependency_table(
    value: Option<&Value>,
    group: impl Into<String>,
    dependencies: &mut Vec<Dependency>,
) {
    let group = group.into();
    let Some(table) = value.and_then(Value::as_table) else {
        return;
    };

    for (name, spec) in table {
        if name.eq_ignore_ascii_case("python") {
            continue;
        }

        dependencies.push(parse_poetry_dependency(name, spec, &group));
    }
}

/// Parse a Poetry dependency entry.
///
/// Poetry entries may be a simple version string or a structured table with
/// alternate package name, extras, markers, source, or path information. The
/// parser extracts the fields that map cleanly to the shared `Dependency`
/// shape and preserves the full TOML value as raw evidence.
fn parse_poetry_dependency(name: &str, spec: &Value, group: &str) -> Dependency {
    match spec {
        Value::String(specifier) => Dependency::new(name, group)
            .with_raw(specifier.clone())
            .with_specifier(specifier.as_str()),
        Value::Table(table) => {
            let mut dependency = Dependency::new(name, group).with_raw(spec.to_string());

            if let Some(package) = table.get("name").and_then(Value::as_str) {
                dependency.name = package.to_string();
            }
            if let Some(version) = table.get("version").and_then(Value::as_str) {
                dependency = dependency.with_specifier(version);
            }
            if let Some(extras) = table.get("extras").and_then(Value::as_array) {
                dependency.extras = extras
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect();
            }
            if let Some(markers) = table.get("markers").and_then(Value::as_str) {
                dependency.marker = Some(markers.to_string());
            }

            dependency
        }
        _ => Dependency::new(name, group).with_raw(spec.to_string()),
    }
}

/// Parse one pip-compatible requirement string.
///
/// This is a permissive parser for graph discovery, not a replacement for pip's
/// requirement parser. It skips comments and options, extracts the likely
/// package name, preserves the original text, and carries markers or extras
/// forward as evidence.
fn parse_requirement_line(line: &str, group: impl Into<String>) -> Option<Dependency> {
    let raw = line.trim();
    if raw.is_empty() || raw.starts_with('#') || raw.starts_with('-') {
        return None;
    }

    let raw = raw
        .split_once(" #")
        .map(|(requirement, _comment)| requirement.trim())
        .unwrap_or(raw);
    let raw = raw
        .strip_prefix("requirement = ")
        .map(str::trim)
        .unwrap_or(raw);

    if let Some(name) = egg_name(raw) {
        return Some(Dependency::new(name, group).with_raw(raw));
    }

    let (requirement, marker) = split_once_trimmed(raw, ';');
    let (requirement, extras) = split_extras(requirement);
    let (name, specifier) = split_name_specifier(&requirement)?;

    let mut dependency = Dependency::new(name, group).with_raw(raw);
    if let Some(specifier) = specifier.filter(|specifier| !specifier.is_empty()) {
        dependency = dependency.with_specifier(specifier);
    }
    dependency.extras = extras;
    dependency.marker = marker.map(str::to_string);

    Some(dependency)
}

/// Get the package name from a VCS URL `#egg=` fragment.
///
/// Older requirements commonly identify VCS dependencies through this fragment.
/// Capturing it lets the graph still create a package node when the requirement
/// starts with a URL instead of a normal package name.
fn egg_name(requirement: &str) -> Option<String> {
    requirement
        .split_once("#egg=")
        .map(|(_, name)| {
            name.split(['&', ';', ' ', '\t'])
                .next()
                .unwrap_or(name)
                .to_string()
        })
        .filter(|name| !name.is_empty())
}

/// Split extras from a package name.
///
/// Extras modify what optional dependency set is requested, but they should not
/// be part of the package node identity. Returning the requirement without the
/// bracketed section lets name/specifier parsing stay simple.
fn split_extras(requirement: &str) -> (String, Vec<String>) {
    let Some(open) = requirement.find('[') else {
        return (requirement.to_string(), Vec::new());
    };
    let Some(close) = requirement[(open + 1)..]
        .find(']')
        .map(|index| open + 1 + index)
    else {
        return (requirement.to_string(), Vec::new());
    };

    let extras = requirement[(open + 1)..close]
        .split(',')
        .map(str::trim)
        .filter(|extra| !extra.is_empty())
        .map(str::to_string)
        .collect();
    (
        format!("{}{}", &requirement[..open], &requirement[(close + 1)..]),
        extras,
    )
}

/// Split the package name and trailing version/source specifier.
///
/// Python requirement strings put the package name first for the common cases
/// this module targets. The remaining text is left as a specifier because it may
/// be a version range, direct reference, or another packaging expression.
fn split_name_specifier(requirement: &str) -> Option<(String, Option<String>)> {
    let requirement = requirement.trim();
    let end = requirement
        .find(|char: char| {
            char.is_whitespace() || matches!(char, '<' | '>' | '=' | '!' | '~' | '@' | ';' | '[')
        })
        .unwrap_or(requirement.len());

    let name = requirement[..end].trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || matches!(char, '-' | '_' | '.'))
    {
        return None;
    }

    let specifier = requirement[end..].trim();
    Some((
        name.to_string(),
        (!specifier.is_empty()).then(|| specifier.to_string()),
    ))
}

/// Split once on a delimiter and trim both sides.
///
/// Requirement markers are separated by semicolons and are useful evidence, but
/// they should not interfere with name or version parsing. This helper keeps
/// that split explicit at the call site.
fn split_once_trimmed(value: &str, delimiter: char) -> (&str, Option<&str>) {
    value
        .split_once(delimiter)
        .map(|(left, right)| (left.trim(), Some(right.trim())))
        .unwrap_or((value, None))
}
