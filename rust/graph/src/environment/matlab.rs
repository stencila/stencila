//! MATLAB environment manifest parsing.
//!
//! MATLAB Package Manager packages store their package definition in
//! `resources/mpackage.json`. This module extracts direct package dependencies
//! from that static JSON manifest without invoking MATLAB or inferring installed
//! MathWorks products.

use std::{fs, path::Path};

use eyre::{Result, WrapErr};
use serde_json::Value;

use super::{Dependency, Ecosystem, EnvironmentManifest};
use crate::ids::WorkspaceRelPath;

/// Parse a MATLAB Package Manager `mpackage.json` manifest.
///
/// The parser records dependencies from the manifest's `dependencies` array.
/// MATLAB package identifiers include a UUID, which is retained as raw evidence
/// and as a PURL qualifier so same-named packages can still be distinguished.
pub(super) fn parse_mpackage_json(
    path: &Path,
    rel: &WorkspaceRelPath,
) -> Result<EnvironmentManifest> {
    let text =
        fs::read_to_string(path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    let value = serde_json::from_str::<Value>(&text)
        .wrap_err_with(|| format!("unable to parse {}", path.display()))?;

    let dependencies = value
        .get("dependencies")
        .and_then(Value::as_array)
        .map(|dependencies| {
            dependencies
                .iter()
                .filter_map(parse_dependency)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(EnvironmentManifest {
        ecosystem: Ecosystem::Matlab,
        rel: rel.clone(),
        dependencies,
        lockfile_names: vec![],
    })
}

/// Parse one MATLAB package dependency object.
fn parse_dependency(value: &Value) -> Option<Dependency> {
    let object = value.as_object()?;
    let name = object.get("name").and_then(Value::as_str)?.trim();
    let id = object.get("id").and_then(Value::as_str)?.trim();
    if name.is_empty() || id.is_empty() {
        return None;
    }

    let mut dependency = Dependency::new(name, "dependencies").with_raw(value.to_string());
    dependency
        .qualifiers
        .push(("uuid".to_string(), id.to_string()));
    if let Some(specifier) = object
        .get("compatibleVersions")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|specifier| !specifier.is_empty())
    {
        dependency.specifier = Some(specifier.to_string());
    }

    Some(dependency)
}
