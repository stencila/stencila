//! Rust environment manifest parsing.
//!
//! Cargo manifests describe direct crate dependencies, workspace dependency
//! declarations, and target-specific dependency tables. This module extracts
//! those declarations from `Cargo.toml` without invoking Cargo or reading the
//! registry.
//!
//! Cargo's resolver, feature unification, target filtering, and lockfile format
//! are richer than the shared environment graph needs for a first pass. The
//! parser records direct declarations and target markers, while `Cargo.lock`
//! remains linked as a sibling evidence file.

use std::{fs, path::Path};

use eyre::{Result, WrapErr};
use toml::Value;

use super::{Dependency, Ecosystem, EnvironmentManifest};
use crate::ids::WorkspaceRelPath;

/// Parse a Cargo manifest.
///
/// The parser reads the dependency tables that can introduce direct crates in a
/// package or workspace manifest. Target-specific tables are retained through
/// the dependency group and marker so consumers can see that a declaration is
/// conditional without evaluating the current compilation target.
pub(super) fn parse_cargo_toml(path: &Path, rel: &WorkspaceRelPath) -> Result<EnvironmentManifest> {
    let text =
        fs::read_to_string(path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    let value = Value::Table(
        text.parse::<toml::Table>()
            .wrap_err_with(|| format!("unable to parse {}", path.display()))?,
    );

    let mut dependencies = Vec::new();
    for group in ["dependencies", "dev-dependencies", "build-dependencies"] {
        push_dependency_table(value.get(group), group, &mut dependencies);
    }

    if let Some(workspace) = value.get("workspace").and_then(Value::as_table) {
        push_dependency_table(
            workspace.get("dependencies"),
            "workspace.dependencies",
            &mut dependencies,
        );
    }

    if let Some(targets) = value.get("target").and_then(Value::as_table) {
        for (target, table) in targets {
            if let Some(target_table) = table.as_table() {
                for group in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    push_dependency_table(
                        target_table.get(group),
                        format!("target.{target}.{group}"),
                        &mut dependencies,
                    );
                }
            }
        }
    }

    Ok(EnvironmentManifest {
        ecosystem: Ecosystem::Rust,
        rel: rel.clone(),
        dependencies,
        lockfile_names: vec!["Cargo.lock"],
    })
}

/// Push dependencies from a TOML dependency table.
///
/// Cargo dependency tables all share the same key/value shape once the table
/// path is known. Passing the group path through keeps parser logic compact and
/// preserves where the declaration appeared in the manifest.
fn push_dependency_table(
    value: Option<&Value>,
    group: impl Into<String>,
    dependencies: &mut Vec<Dependency>,
) {
    let group = group.into();
    let Some(table) = value.and_then(Value::as_table) else {
        return;
    };

    dependencies.extend(
        table
            .iter()
            .filter_map(|(name, value)| parse_dependency(name, value, &group)),
    );
}

/// Parse one Cargo dependency entry.
///
/// Cargo dependencies can be plain version strings or inline tables with
/// renamed packages, features, workspace inheritance, paths, Git sources, and
/// target-specific conditions. This helper extracts the portable pieces and
/// keeps the full TOML value as raw evidence for anything more specific.
fn parse_dependency(name: &str, value: &Value, group: &str) -> Option<Dependency> {
    match value {
        Value::String(specifier) => Some(
            Dependency::new(name, group)
                .with_raw(specifier.clone())
                .with_specifier(specifier.as_str()),
        ),
        Value::Table(table) => {
            let package_name = table.get("package").and_then(Value::as_str).unwrap_or(name);
            let mut dependency = Dependency::new(package_name, group).with_raw(value.to_string());

            if let Some(version) = table.get("version").and_then(Value::as_str) {
                dependency = dependency.with_specifier(version);
            } else if table
                .get("workspace")
                .and_then(Value::as_bool)
                .is_some_and(|workspace| workspace)
            {
                dependency = dependency.with_specifier("workspace");
            }

            if let Some(features) = table.get("features").and_then(Value::as_array) {
                dependency.extras = features
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect();
            }
            if let Some(target) = group.strip_prefix("target.").and_then(|rest| {
                rest.rsplit_once('.')
                    .map(|(target, _dependency_kind)| target.to_string())
            }) {
                dependency.marker = Some(target);
            }

            Some(dependency)
        }
        _ => None,
    }
}
