//! Julia environment manifest parsing.
//!
//! Julia projects declare direct package dependencies in `Project.toml` tables
//! such as `[deps]`, `[weakdeps]`, and `[extras]`. This module extracts those
//! package declarations without invoking Julia or expanding `Manifest.toml`.
//!
//! Julia package names are not globally unique without UUIDs, so the parser
//! carries dependency UUIDs into package URLs as `uuid` qualifiers. Lockfiles
//! remain linked as sibling evidence files, consistent with the shared
//! environment graph model used for other ecosystems.

use std::{collections::BTreeSet, fs, path::Path};

use eyre::{Result, WrapErr};
use toml::Value;

use super::{Dependency, Ecosystem, EnvironmentManifest, is_julia_lockfile_name};
use crate::ids::WorkspaceRelPath;

/// Parse a Julia project manifest.
///
/// The parser records author-declared direct packages from Julia's standard
/// dependency tables. Compatibility constraints are preserved as specifiers
/// when they correspond to one of those package declarations; the Julia runtime
/// constraint itself is skipped because it describes the interpreter, not a
/// package node.
pub(super) fn parse_project_toml(
    path: &Path,
    rel: &WorkspaceRelPath,
) -> Result<EnvironmentManifest> {
    let text =
        fs::read_to_string(path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    let value = Value::Table(
        text.parse::<toml::Table>()
            .wrap_err_with(|| format!("unable to parse {}", path.display()))?,
    );

    let compat = value.get("compat").and_then(Value::as_table);
    let mut dependencies = Vec::new();
    for group in ["deps", "weakdeps", "extras"] {
        push_dependency_table(value.get(group), group, compat, &mut dependencies);
    }

    Ok(EnvironmentManifest {
        ecosystem: Ecosystem::Julia,
        rel: rel.clone(),
        dependencies,
        lockfile_names: julia_lockfile_names(path),
    })
}

/// Push dependencies from a Julia project dependency table.
///
/// Julia dependency tables map package names to UUID strings. The UUID is part
/// of package identity, so it is retained as both raw evidence and a PURL
/// qualifier.
fn push_dependency_table(
    value: Option<&Value>,
    group: &str,
    compat: Option<&toml::Table>,
    dependencies: &mut Vec<Dependency>,
) {
    let Some(table) = value.and_then(Value::as_table) else {
        return;
    };

    dependencies.extend(table.iter().filter_map(|(name, value)| {
        let uuid = value.as_str()?.trim();
        if uuid.is_empty() {
            return None;
        }

        let mut dependency = Dependency::new(name, group).with_raw(uuid);
        dependency
            .qualifiers
            .push(("uuid".to_string(), uuid.to_string()));
        if let Some(specifier) = compat
            .and_then(|compat| compat.get(name))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|specifier| !specifier.is_empty())
        {
            dependency.specifier = Some(specifier.to_string());
        }

        Some(dependency)
    }));
}

/// Collect conventional and versioned sibling Julia lockfile names.
///
/// Versioned manifests include the Julia version in the filename, so they need
/// to be discovered from the manifest's directory rather than hard-coded.
fn julia_lockfile_names(path: &Path) -> Vec<String> {
    let mut names = BTreeSet::from([
        "Manifest.toml".to_string(),
        "JuliaManifest.toml".to_string(),
    ]);

    if let Some(parent) = path.parent() {
        push_sibling_julia_lockfile_names(parent, &mut names);
    }

    names.into_iter().collect()
}

/// Add versioned Julia lockfile names found beside the project file.
fn push_sibling_julia_lockfile_names(parent: &Path, names: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };

    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|file_type| file_type.is_file()) {
            continue;
        }

        let name = entry.file_name();
        if let Some(name) = name.to_str()
            && is_julia_lockfile_name(name)
        {
            names.insert(name.to_string());
        }
    }
}
