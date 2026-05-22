//! Shared software package identity helpers.
//!
//! Package nodes can be discovered from source-code imports or from environment
//! manifests. Keeping their id and node construction in one place prevents the
//! two collectors from creating incompatible graph nodes for the same package.

use std::fmt::Write as _;

use stencila_schema::{Primitive, PropertyValue, PropertyValueOrString, SoftwareSourceCode};

use crate::ids::LocalGraphId;

/// A package imported by source code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageFact {
    /// Package ecosystem or registry namespace, such as `pypi`, `cran`, or `npm`.
    pub ecosystem: String,

    /// Package name as detected from source.
    pub name: String,
}

impl PackageFact {
    /// Create a source-code package fact.
    pub fn new(ecosystem: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            ecosystem: ecosystem.into(),
            name: name.into(),
        }
    }
}

/// Create the graph-local id for a package concept.
///
/// Versions are intentionally excluded: source-code imports are unversioned, and
/// declared versions belong on requirement evidence rather than on package
/// identity. Qualifiers remain part of identity because ecosystems such as Julia
/// use UUIDs to disambiguate packages with the same name.
pub(crate) fn package_id(ecosystem: &str, name: &str, qualifiers: &[(String, String)]) -> String {
    LocalGraphId::package(&package_id_component(ecosystem, name, qualifiers))
}

/// Create a package node that is compatible across code and environment passes.
pub(crate) fn package_node(
    ecosystem: &str,
    name: &str,
    qualifiers: &[(String, String)],
) -> SoftwareSourceCode {
    let canonical_name = package_name_for_identity(ecosystem, name);
    let mut package = SoftwareSourceCode::new(
        canonical_name,
        package_programming_language(ecosystem).to_string(),
    );
    package.id = Some(package_id(ecosystem, name, qualifiers));
    if package_has_purl(ecosystem) {
        package.options.identifiers = Some(vec![property_value_identifier(
            "purl",
            package_identity_purl(ecosystem, name, qualifiers),
        )]);
    }

    package
}

/// Build an unversioned PURL for package identity.
pub(crate) fn package_identity_purl(
    ecosystem: &str,
    name: &str,
    qualifiers: &[(String, String)],
) -> String {
    package_purl(ecosystem, name, None, qualifiers)
}

/// Build a PURL for a declared package requirement.
pub(crate) fn package_requirement_purl(
    ecosystem: &str,
    name: &str,
    version: Option<&str>,
    qualifiers: &[(String, String)],
) -> String {
    package_purl(ecosystem, name, version, qualifiers)
}

/// Normalize Python package names as used by Python package indexes.
///
/// Python package names compare case-insensitively with runs of `.`, `_`, and
/// `-` treated as equivalent. Applying that normalization before package
/// identity creation avoids separate graph nodes for spelling variants of the
/// same package.
pub(crate) fn normalize_python_package_name(name: &str) -> String {
    let mut normalized = String::new();
    let mut previous_dash = false;
    for char in name.chars().flat_map(char::to_lowercase) {
        if matches!(char, '-' | '_' | '.') {
            if !previous_dash {
                normalized.push('-');
                previous_dash = true;
            }
        } else {
            normalized.push(char);
            previous_dash = false;
        }
    }

    normalized
}

fn package_id_component(ecosystem: &str, name: &str, qualifiers: &[(String, String)]) -> String {
    let mut component = format!("{ecosystem}/{}", package_name_for_identity(ecosystem, name));

    if !qualifiers.is_empty() {
        component.push('?');
        component.push_str(
            &qualifiers
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("&"),
        );
    }

    component
}

fn package_purl(
    ecosystem: &str,
    name: &str,
    version: Option<&str>,
    qualifiers: &[(String, String)],
) -> String {
    let mut purl = format!("pkg:{}/{}", ecosystem, purl_package_name(ecosystem, name));

    if let Some(version) = version {
        purl.push('@');
        purl.push_str(&encode_purl_component(version));
    }

    if !qualifiers.is_empty() {
        purl.push('?');
        purl.push_str(
            &qualifiers
                .iter()
                .map(|(key, value)| {
                    format!(
                        "{}={}",
                        encode_purl_component(key),
                        encode_purl_component(value)
                    )
                })
                .collect::<Vec<_>>()
                .join("&"),
        );
    }

    purl
}

/// Normalize and encode a package name for PURL use.
///
/// Package name rules differ by ecosystem. Python package indexes normalize
/// separators and case, while scoped npm packages need to preserve their
/// `@scope/name` structure across PURL path segments.
fn purl_package_name(ecosystem: &str, name: &str) -> String {
    let name = package_name_for_identity(ecosystem, name);

    match ecosystem {
        "npm" if name.starts_with('@') => {
            let Some((scope, package)) = name.split_once('/') else {
                return encode_purl_component(&name);
            };
            format!(
                "{}/{}",
                encode_purl_component(scope),
                encode_purl_component(package)
            )
        }
        _ => encode_purl_component(&name),
    }
}

fn package_name_for_identity(ecosystem: &str, name: &str) -> String {
    match ecosystem {
        "pypi" => normalize_python_package_name(name),
        _ => name.to_string(),
    }
}

fn package_programming_language(ecosystem: &str) -> &'static str {
    match ecosystem {
        "cargo" => "Rust",
        "cran" => "R",
        "julia" => "Julia",
        "matlab" => "MATLAB",
        "pypi" => "Python",
        "node" | "npm" => "JavaScript",
        _ => "Unknown",
    }
}

fn package_has_purl(ecosystem: &str) -> bool {
    !matches!(ecosystem, "node")
}

/// Percent-encode a PURL component while keeping PURL separators external.
fn encode_purl_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            byte => {
                let _ = write!(&mut encoded, "%{byte:02X}");
            }
        }
    }

    encoded
}

fn property_value_identifier(
    property_id: impl Into<String>,
    value: impl Into<String>,
) -> PropertyValueOrString {
    let mut identifier = PropertyValue::new(Primitive::String(value.into()));
    identifier.property_id = Some(property_id.into());
    PropertyValueOrString::PropertyValue(identifier)
}
