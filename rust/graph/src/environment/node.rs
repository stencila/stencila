//! Node.js environment manifest parsing.
//!
//! Node.js projects declare direct dependencies in `package.json`, while
//! package-manager-specific lockfiles sit next to that manifest. This parser
//! reads only the manifest because it is the stable author-authored declaration
//! that maps cleanly into environment and package graph nodes.
//!
//! Lockfiles such as `package-lock.json`, `pnpm-lock.yaml`, and `yarn.lock`
//! have different structures and transitive dependency semantics. The shared
//! environment pass links those files as reproducibility evidence instead of
//! expanding them here.

use std::{fs, path::Path};

use eyre::{Result, WrapErr};
use serde_json::Value;

use super::{Dependency, Ecosystem, EnvironmentManifest};
use crate::ids::WorkspaceRelPath;

/// Parse an npm-compatible `package.json` manifest.
///
/// The parser collects the common dependency groups that can affect execution
/// or development workflows. It does not interpret npm range syntax because
/// range resolution depends on package manager behavior and registry state.
pub(super) fn parse_package_json(
    path: &Path,
    rel: &WorkspaceRelPath,
) -> Result<EnvironmentManifest> {
    let text =
        fs::read_to_string(path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    let value = serde_json::from_str::<Value>(&text)
        .wrap_err_with(|| format!("unable to parse {}", path.display()))?;

    let mut dependencies = Vec::new();
    for group in [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        push_dependency_object(value.get(group), group, &mut dependencies);
    }
    push_bundle_dependencies(value.get("bundleDependencies"), &mut dependencies);
    push_bundle_dependencies(value.get("bundledDependencies"), &mut dependencies);

    Ok(EnvironmentManifest {
        ecosystem: Ecosystem::Node,
        rel: rel.clone(),
        dependencies,
        lockfile_names: vec![
            "package-lock.json",
            "pnpm-lock.yaml",
            "yarn.lock",
            "bun.lockb",
        ],
    })
}

/// Push dependencies from a package dependency object.
///
/// Most dependency groups are JSON objects keyed by package name with a version,
/// range, workspace reference, URL, or file reference as the value. The raw
/// value is preserved so non-version specifiers remain visible in evidence.
fn push_dependency_object(value: Option<&Value>, group: &str, dependencies: &mut Vec<Dependency>) {
    let Some(object) = value.and_then(Value::as_object) else {
        return;
    };

    dependencies.extend(object.iter().map(|(name, value)| {
        let mut dependency = Dependency::new(name, group);
        if let Some(specifier) = value.as_str() {
            dependency = dependency.with_raw(specifier).with_specifier(specifier);
        } else {
            dependency = dependency.with_raw(value.to_string());
        }

        dependency
    }));
}

/// Push dependencies from bundled dependency arrays.
///
/// Bundled dependency arrays list package names without version specifiers.
/// They still represent author intent about package inclusion, so they are
/// included as direct declarations with group metadata.
fn push_bundle_dependencies(value: Option<&Value>, dependencies: &mut Vec<Dependency>) {
    let Some(array) = value.and_then(Value::as_array) else {
        return;
    };

    dependencies.extend(
        array
            .iter()
            .filter_map(Value::as_str)
            .map(|name| Dependency::new(name, "bundleDependencies")),
    );
}
