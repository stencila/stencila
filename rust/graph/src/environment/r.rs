//! R environment manifest parsing.
//!
//! R packages declare package dependencies in `DESCRIPTION` fields such as
//! `Depends`, `Imports`, `Suggests`, and `LinkingTo`. This module extracts
//! those direct declarations while preserving version constraints as evidence.
//!
//! R environment restoration tools such as renv add lockfile-level
//! reproducibility, but their lockfiles are not expanded here. The shared
//! environment pass links `renv.lock` as a sibling evidence file so the graph
//! can show that a reproducible environment description exists.

use std::{collections::BTreeMap, fs, path::Path};

use eyre::{Result, WrapErr};

use super::{Dependency, Ecosystem, EnvironmentManifest};
use crate::ids::WorkspaceRelPath;

/// Parse an R `DESCRIPTION` manifest.
///
/// The parser follows the Debian-control-style field format used by R package
/// metadata, including continuation lines. It records direct package
/// declarations from the fields most likely to affect package installation or
/// execution.
pub(super) fn parse_description(
    path: &Path,
    rel: &WorkspaceRelPath,
) -> Result<EnvironmentManifest> {
    let text =
        fs::read_to_string(path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    let fields = description_fields(&text);

    let mut dependencies = Vec::new();
    for group in ["Depends", "Imports", "Suggests", "LinkingTo"] {
        if let Some(value) = fields.get(group) {
            dependencies.extend(
                value
                    .split(',')
                    .filter_map(|requirement| parse_description_requirement(requirement, group)),
            );
        }
    }

    Ok(EnvironmentManifest {
        ecosystem: Ecosystem::R,
        rel: rel.clone(),
        dependencies,
        lockfile_names: vec!["renv.lock".to_string()],
    })
}

/// Parse DESCRIPTION fields, including continuation lines.
///
/// DESCRIPTION files fold long field values onto indented continuation lines.
/// Normalizing those lines into one field map makes dependency parsing
/// independent of line wrapping chosen by package authors or tooling.
fn description_fields(text: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::<String, String>::new();
    let mut current_key: Option<String> = None;

    for line in text.lines() {
        if line.starts_with([' ', '\t']) {
            if let Some(key) = &current_key {
                let value = fields.entry(key.clone()).or_default();
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(line.trim());
            }
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            current_key = None;
            continue;
        };

        let key = key.trim().to_string();
        fields.insert(key.clone(), value.trim().to_string());
        current_key = Some(key);
    }

    fields
}

/// Parse one DESCRIPTION dependency declaration.
///
/// R dependency entries are comma-separated and may include a parenthesized
/// version constraint. The package name becomes graph identity, while the
/// constraint is preserved as a specifier rather than resolved against the
/// current R installation.
fn parse_description_requirement(requirement: &str, group: &str) -> Option<Dependency> {
    let requirement = requirement.trim();
    if requirement.is_empty() {
        return None;
    }

    let name_end = requirement
        .find(|char: char| char.is_whitespace() || char == '(')
        .unwrap_or(requirement.len());
    let name = requirement[..name_end].trim();
    if name.is_empty() || name == "R" {
        return None;
    }

    let mut dependency = Dependency::new(name, group).with_raw(requirement);
    if let Some(open) = requirement.find('(')
        && let Some(close) = requirement[(open + 1)..]
            .find(')')
            .map(|index| open + 1 + index)
    {
        dependency = dependency.with_specifier(requirement[(open + 1)..close].trim());
    }

    Some(dependency)
}
