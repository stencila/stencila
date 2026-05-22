//! Static extraction of computational environment metadata from workspace files.
//!
//! This module parses common manifest files without executing package managers.
//! It records direct dependency declarations and links lockfiles as supporting
//! environment files, leaving full lockfile expansion to a later pass.
//!
//! Environment capture sits between filesystem inventory and code/document graph
//! extraction. The workspace graph already knows that files exist, while code
//! analysis can discover imports and runtime calls. This module adds the
//! author-declared environment layer: manifests state what software should be
//! available, and lockfiles provide reproducibility evidence even when this pass
//! does not interpret every transitive package.
//!
//! The implementation deliberately favors deterministic static facts over
//! package-manager fidelity. Running `pip`, `npm`, `cargo`, or R tooling during
//! graph construction would make graph output depend on host state, network
//! access, package indexes, and tool versions. Instead, each ecosystem parser
//! returns a normalized manifest summary that can be projected into Schema graph
//! nodes using one shared graph shape.

mod julia;
mod node;
mod python;
mod r;
mod rust;

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use eyre::{Result, WrapErr};
use sha2::{Digest, Sha256};
use stencila_schema::{
    GraphEdgeKind, GraphEvidence, Node as SchemaNode, Object, Primitive, PropertyValue,
    PropertyValueOrString, SoftwareApplication, SoftwareApplicationOrSoftwareSourceCodeOrString,
    SoftwareSourceCode, StringOrNumber,
};

use crate::{
    GraphBuilder, evidence,
    ids::{LocalGraphId, WorkspaceRelPath},
};

/// A manifest file parsed into a declared computational environment.
///
/// Parsers use this intermediate value so ecosystem-specific syntax does not
/// leak into graph construction. Keeping the graph projection centralized means
/// Python, Node.js, Rust, and R manifests produce the same environment, package,
/// and lockfile relationship pattern even though their source formats differ
/// substantially.
#[derive(Debug, Clone)]
struct EnvironmentManifest {
    /// Ecosystem described by the manifest.
    ///
    /// The ecosystem controls labels, PURL package type, and the programming
    /// language attached to generated package nodes. This keeps parser modules
    /// small while preserving ecosystem-specific identity in graph output.
    ecosystem: Ecosystem,

    /// Workspace-relative manifest path.
    ///
    /// The manifest path scopes the environment id. Workspaces often contain
    /// multiple projects or nested packages, so one ecosystem can legitimately
    /// appear more than once in a single graph.
    rel: WorkspaceRelPath,

    /// Direct dependency declarations found in the manifest.
    ///
    /// This pass records author-declared direct requirements only. Transitive
    /// dependency closure is left to lockfile or resolver-aware passes because
    /// interpreting it accurately requires ecosystem-specific semantics.
    dependencies: Vec<Dependency>,

    /// Lockfile names that should be associated with this manifest when present.
    ///
    /// Lockfiles are linked as sibling files rather than parsed here so the
    /// graph can still show reproducibility evidence without pretending that a
    /// generic manifest pass understands every lockfile dialect.
    lockfile_names: Vec<String>,
}

/// Supported direct-dependency ecosystems.
///
/// This enum is intentionally about manifest ecosystems, not execution kernels
/// or programming languages. For example, a Node.js environment may include
/// TypeScript tooling, and a Python project may declare compiled extensions, but
/// the manifest parser still needs one stable package namespace for PURLs and
/// graph ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ecosystem {
    Python,
    Node,
    Rust,
    R,
    Julia,
}

impl Ecosystem {
    /// Stable lowercase ecosystem label for evidence and ids.
    ///
    /// Graph-local ids should be easy to scan and stable across display-name
    /// changes. The slug is the compact form used in environment ids and
    /// evidence details.
    fn slug(self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::Node => "node",
            Self::Rust => "rust",
            Self::R => "r",
            Self::Julia => "julia",
        }
    }

    /// Human-readable ecosystem label.
    ///
    /// Schema node names are user-facing enough that they should not expose the
    /// lowercase id spelling. Keeping labels here avoids duplicating display
    /// strings in each parser.
    fn name(self) -> &'static str {
        match self {
            Self::Python => "Python",
            Self::Node => "Node.js",
            Self::Rust => "Rust",
            Self::R => "R",
            Self::Julia => "Julia",
        }
    }

    /// PURL package type for packages in this ecosystem.
    ///
    /// Package URLs give downstream consumers a familiar cross-ecosystem
    /// identifier without requiring Stencila to mint package identifiers of its
    /// own. The type segment is the part that varies by ecosystem.
    fn purl_type(self) -> &'static str {
        match self {
            Self::Python => "pypi",
            Self::Node => "npm",
            Self::Rust => "cargo",
            Self::R => "cran",
            Self::Julia => "julia",
        }
    }

    /// Programming language used for package `SoftwareSourceCode` nodes.
    ///
    /// Dependencies are represented as `SoftwareSourceCode` because packages
    /// are usually source or library artifacts rather than installed
    /// applications. The language label gives consumers a coarse way to group
    /// package nodes without parsing the PURL.
    fn programming_language(self) -> &'static str {
        match self {
            Self::Python => "Python",
            Self::Node => "JavaScript",
            Self::Rust => "Rust",
            Self::R => "R",
            Self::Julia => "Julia",
        }
    }
}

/// A direct package requirement read from an environment manifest.
///
/// This is intentionally lossy but structured. Manifest syntaxes can express
/// ranges, extras, target conditions, source URLs, workspace inheritance, and
/// tool-specific shapes. The graph needs enough structure to identify the
/// package and preserve the declaration, while keeping unresolved semantics in
/// evidence details for later passes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Dependency {
    /// Package name.
    ///
    /// This is the canonical package name as far as the parser can determine
    /// from the manifest entry. Ecosystem-level normalization happens later
    /// when building PURLs.
    pub(super) name: String,

    /// Dependency group, such as `dependencies`, `devDependencies`, or `Imports`.
    ///
    /// Groups preserve author intent about how a package is used without
    /// requiring the graph edge kind taxonomy to encode every ecosystem's
    /// dependency classes.
    pub(super) group: String,

    /// Raw requirement text or manifest value.
    ///
    /// The raw declaration is kept as evidence because parsers may not fully
    /// understand every specifier form, but consumers should still be able to
    /// inspect exactly what the manifest said.
    pub(super) raw: Option<String>,

    /// Version constraint or manifest specifier, when distinct from `raw`.
    ///
    /// This field captures the part most useful for quick filtering and display
    /// while allowing the raw field to carry the full declaration.
    pub(super) specifier: Option<String>,

    /// Exact version when the manifest declares one without a range.
    ///
    /// Exact versions can safely be folded into package PURLs and Schema
    /// `version` fields. Range expressions stay out of package identity so that
    /// `requests>=2` and `requests<3` still point at the same package concept.
    pub(super) exact_version: Option<String>,

    /// Extras, features, or similar package modifiers.
    ///
    /// These are intentionally grouped across ecosystems because Python extras,
    /// Cargo features, and similar modifiers have different semantics but play
    /// the same graph role: they refine a dependency declaration.
    pub(super) extras: Vec<String>,

    /// Environment marker or target condition.
    ///
    /// Conditional requirements affect whether a dependency applies on a given
    /// platform or interpreter. The condition is preserved as evidence rather
    /// than evaluated against the current host.
    pub(super) marker: Option<String>,

    /// PURL qualifiers for ecosystem-specific package identity.
    ///
    /// Some ecosystems need a package identifier in addition to the name. For
    /// example, Julia package identity includes a UUID, represented in PURLs as
    /// a qualifier rather than as part of the package name.
    pub(super) qualifiers: Vec<(String, String)>,
}

impl Dependency {
    /// Create a dependency declaration.
    ///
    /// Parsers start from the two fields shared by every dependency syntax and
    /// then layer raw text, specifiers, extras, or markers as those details are
    /// discovered.
    pub(super) fn new(name: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            group: group.into(),
            raw: None,
            specifier: None,
            exact_version: None,
            extras: Vec::new(),
            marker: None,
            qualifiers: Vec::new(),
        }
    }

    /// Set raw requirement text.
    ///
    /// Builder-style setters keep parser code compact while making it obvious
    /// which parts of a declaration are understood versus merely preserved.
    pub(super) fn with_raw(mut self, raw: impl Into<String>) -> Self {
        self.raw = Some(raw.into());
        self
    }

    /// Set a version or source specifier.
    ///
    /// This also extracts an exact version when the specifier is safe to use as
    /// package identity. Doing that once here keeps ecosystem parsers from
    /// duplicating range-detection heuristics.
    pub(super) fn with_specifier(mut self, specifier: impl Into<String>) -> Self {
        let specifier = specifier.into();
        self.exact_version = exact_version(&specifier);
        self.specifier = Some(specifier);
        self
    }
}

/// Whether a workspace file should receive environment-file treatment.
///
/// Workspace file nodes call this before adding digest identifiers. The helper
/// includes both manifests and lockfiles because either can materially affect a
/// computational environment even when only manifests create environment nodes.
pub(crate) fn is_environment_file(rel: &WorkspaceRelPath) -> bool {
    let name = file_name(rel);
    is_manifest_name(name) || is_lockfile_name(name)
}

/// Create a SHA-256 identifier for an environment manifest or lockfile.
///
/// Digests make lockfile and manifest evidence content-addressable without
/// embedding file contents in the graph. Failures are ignored by returning
/// `None` because file metadata collection should not fail solely because an
/// optional digest could not be read.
pub(crate) fn file_digest_identifier(
    path: &Path,
    rel: &WorkspaceRelPath,
) -> Option<PropertyValueOrString> {
    if !is_environment_file(rel) {
        return None;
    }

    sha256_file(path)
        .ok()
        .map(|digest| property_value_identifier("sha256", digest))
}

/// Add environment metadata discovered from a workspace file.
///
/// The workspace walker calls this for every file so manifest detection stays
/// local to the environment module. Parse errors are configurable: inventory
/// callers usually want a best-effort graph, while CI or validation workflows
/// may prefer strict failure when a declared environment cannot be read.
pub(crate) fn add_environment_from_file(
    builder: &mut GraphBuilder,
    path: &Path,
    rel: &WorkspaceRelPath,
    file_id_for_rel: impl Fn(&WorkspaceRelPath) -> Option<String>,
    fail_on_environment_error: bool,
) -> Result<()> {
    let manifest = match parse_manifest(path, rel) {
        Ok(Some(manifest)) => manifest,
        Ok(None) => return Ok(()),
        Err(error) if fail_on_environment_error => {
            return Err(error).wrap_err_with(|| {
                format!("unable to analyze environment file {}", path.display())
            });
        }
        Err(..) => return Ok(()),
    };

    add_manifest_environment(builder, manifest, file_id_for_rel)
}

/// Parse a supported manifest file.
///
/// Dispatch is based on basename rather than extension because environment
/// conventions use fixed filenames. This also avoids accidentally parsing
/// unrelated TOML or JSON files as package manifests.
fn parse_manifest(path: &Path, rel: &WorkspaceRelPath) -> Result<Option<EnvironmentManifest>> {
    let name = file_name(rel);
    let manifest = match name {
        "pyproject.toml" => python::parse_pyproject(path, rel)?,
        "requirements.txt" | "requirements.in" => python::parse_requirements(path, rel)?,
        "package.json" => node::parse_package_json(path, rel)?,
        "Cargo.toml" => rust::parse_cargo_toml(path, rel)?,
        "DESCRIPTION" => r::parse_description(path, rel)?,
        "Project.toml" | "JuliaProject.toml" => julia::parse_project_toml(path, rel)?,
        _ => return Ok(None),
    };

    Ok(Some(manifest))
}

/// Add a parsed manifest as environment, package, and lockfile graph nodes/edges.
///
/// Every manifest is represented by the same graph pattern: the manifest is
/// derived into an environment node, package nodes are part of that environment,
/// and sibling lockfiles are referenced by it. The uniform shape is more
/// important here than modeling each package manager's full ontology because
/// consumers can query one relationship pattern across ecosystems.
fn add_manifest_environment(
    builder: &mut GraphBuilder,
    manifest: EnvironmentManifest,
    file_id_for_rel: impl Fn(&WorkspaceRelPath) -> Option<String>,
) -> Result<()> {
    let manifest_file_id = LocalGraphId::file(&manifest.rel);
    let environment_id = LocalGraphId::environment(manifest.ecosystem.slug(), &manifest.rel);

    let requirements = manifest
        .dependencies
        .iter()
        .map(|dependency| {
            SoftwareApplicationOrSoftwareSourceCodeOrString::String(package_purl(
                manifest.ecosystem,
                dependency,
            ))
        })
        .collect::<Vec<_>>();

    let mut environment = SoftwareApplication::new(format!(
        "{} environment declared by {}",
        manifest.ecosystem.name(),
        manifest.rel.as_str()
    ));
    environment.id = Some(environment_id.clone());
    environment.options.path = Some(manifest.rel.as_str().to_string());
    if !requirements.is_empty() {
        environment.options.software_requirements = Some(requirements);
    }

    builder.add_schema_node(
        environment_id.clone(),
        SchemaNode::SoftwareApplication(environment),
    );
    builder.add_edge_with_evidence(
        manifest_file_id,
        &environment_id,
        GraphEdgeKind::DerivedInto,
        vec![evidence::static_analysis()],
    );

    for dependency in &manifest.dependencies {
        let package_id = LocalGraphId::package(&package_purl(manifest.ecosystem, dependency));
        let mut package = SoftwareSourceCode::new(
            dependency.name.clone(),
            manifest.ecosystem.programming_language().to_string(),
        );
        package.id = Some(package_id.clone());
        package.options.identifiers = Some(vec![property_value_identifier(
            "purl",
            package_purl(manifest.ecosystem, dependency),
        )]);
        if let Some(version) = &dependency.exact_version {
            package.version = Some(StringOrNumber::String(version.clone()));
        }

        builder.add_schema_node(package_id.clone(), SchemaNode::SoftwareSourceCode(package));
        builder.add_edge_with_evidence(
            package_id,
            &environment_id,
            GraphEdgeKind::PartOf,
            declared_static_analysis_evidence(&manifest, dependency),
        );
    }

    for lockfile in associated_lockfiles(&manifest) {
        if let Some(lockfile_id) = file_id_for_rel(&lockfile) {
            builder.add_edge_with_evidence(
                lockfile_id,
                &environment_id,
                GraphEdgeKind::ReferencedBy,
                vec![evidence::static_analysis()],
            );
        }
    }

    Ok(())
}

/// Direct manifest filenames.
///
/// The list is deliberately small and conventional. Adding a new filename here
/// means this module is prepared to create environment graph nodes for it, not
/// merely attach file digests.
fn is_manifest_name(name: &str) -> bool {
    matches!(
        name,
        "pyproject.toml"
            | "requirements.txt"
            | "requirements.in"
            | "package.json"
            | "Cargo.toml"
            | "DESCRIPTION"
            | "Project.toml"
            | "JuliaProject.toml"
    )
}

/// Lockfile filenames that are recorded but not expanded in this pass.
///
/// Lockfiles are environment evidence even when they are not the source of
/// direct dependency declarations. They receive file digests and may be linked
/// to a sibling manifest's environment node.
fn is_lockfile_name(name: &str) -> bool {
    matches!(
        name,
        "Cargo.lock"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "bun.lockb"
            | "uv.lock"
            | "poetry.lock"
            | "Pipfile.lock"
            | "renv.lock"
    ) || is_julia_lockfile_name(name)
}

/// Julia lockfile filenames.
///
/// Julia supports conventional `Manifest.toml` files, alternative
/// `JuliaManifest.toml` files, and Julia-version-specific lockfiles such as
/// `Manifest-v1.11.toml`.
fn is_julia_lockfile_name(name: &str) -> bool {
    matches!(name, "Manifest.toml" | "JuliaManifest.toml")
        || versioned_julia_lockfile_name(name, "Manifest-v")
        || versioned_julia_lockfile_name(name, "JuliaManifest-v")
}

/// Whether a filename follows a Julia versioned manifest pattern.
fn versioned_julia_lockfile_name(name: &str, prefix: &str) -> bool {
    let Some(version) = name
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(".toml"))
    else {
        return false;
    };

    !version.is_empty()
        && version
            .chars()
            .all(|char| char.is_ascii_digit() || char == '.')
}

/// Workspace-relative lockfile siblings associated with a manifest.
///
/// Most package managers place lockfiles next to their manifest. Looking only at
/// siblings avoids surprising cross-project links in workspaces with nested
/// packages or multiple ecosystems.
fn associated_lockfiles(manifest: &EnvironmentManifest) -> Vec<WorkspaceRelPath> {
    manifest
        .lockfile_names
        .iter()
        .filter_map(|name| sibling_rel(&manifest.rel, name))
        .collect()
}

/// Return a sibling path relative to the workspace root.
///
/// This keeps lockfile lookup in the same normalized path space as file graph
/// ids. It also handles root-level manifests without special-casing them in the
/// caller.
fn sibling_rel(rel: &WorkspaceRelPath, name: &str) -> Option<WorkspaceRelPath> {
    let mut path = PathBuf::new();
    if let Some(parent) = rel.parent()
        && parent.as_str() != "."
    {
        path.push(parent.as_str());
    }
    path.push(name);

    WorkspaceRelPath::from_relative_path(&path).ok()
}

/// File name from a workspace-relative path.
///
/// Environment conventions are based on basenames, but graph ids and schema
/// paths retain full workspace-relative paths. This helper keeps that split
/// explicit.
fn file_name(rel: &WorkspaceRelPath) -> &str {
    rel.as_str()
        .rsplit_once('/')
        .map(|(_, name)| name)
        .unwrap_or_else(|| rel.as_str())
}

/// Build a PURL for a dependency.
///
/// PURLs provide stable package identifiers that can be used both as Schema
/// identifiers and graph-local package node keys. Only exact versions are
/// included so range declarations do not fragment one package into many nodes.
fn package_purl(ecosystem: Ecosystem, dependency: &Dependency) -> String {
    let mut purl = format!(
        "pkg:{}/{}",
        ecosystem.purl_type(),
        purl_package_name(ecosystem, &dependency.name)
    );

    if let Some(version) = &dependency.exact_version {
        purl.push('@');
        purl.push_str(&encode_purl_component(version));
    }

    if !dependency.qualifiers.is_empty() {
        purl.push('?');
        purl.push_str(
            &dependency
                .qualifiers
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
fn purl_package_name(ecosystem: Ecosystem, name: &str) -> String {
    let name = match ecosystem {
        Ecosystem::Python => normalize_python_package_name(name),
        _ => name.to_string(),
    };

    match ecosystem {
        Ecosystem::Node if name.starts_with('@') => {
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

/// Normalize Python package names as used by Python package indexes.
///
/// Python package names compare case-insensitively with runs of `.`, `_`, and
/// `-` treated as equivalent. Applying that normalization before PURL creation
/// avoids separate graph nodes for spelling variants of the same package.
fn normalize_python_package_name(name: &str) -> String {
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

/// Percent-encode a PURL component while keeping common PURL separators intact.
///
/// This is intentionally narrower than graph-id encoding: PURL components need
/// URI-safe spelling, then the full PURL is encoded again when placed inside a
/// graph-local id.
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

/// Treat common equality specifiers as exact package versions.
///
/// Exact versions can become package identity; ranges and source locators
/// cannot. This heuristic is conservative because a false exact version would
/// create misleading package nodes, while a missed exact version only leaves
/// the version in evidence details.
fn exact_version(specifier: &str) -> Option<String> {
    let specifier = specifier.trim();
    let version = specifier
        .strip_prefix("==")
        .or_else(|| specifier.strip_prefix('='))
        .map(str::trim)?;

    if version.is_empty()
        || version.contains(['*', '<', '>', '=', '!', '~', '^', ',', '|', '&'])
        || version.starts_with("git")
        || version.contains("://")
    {
        None
    } else {
        Some(version.to_string())
    }
}

/// Evidence for a declared relationship discovered by static analysis.
///
/// Dependency edges need both meanings: the manifest declares the package, and
/// this module discovered that declaration by static analysis. Attaching the
/// same structured details to both evidence entries makes either provenance
/// path self-contained for downstream consumers.
fn declared_static_analysis_evidence(
    manifest: &EnvironmentManifest,
    dependency: &Dependency,
) -> Vec<GraphEvidence> {
    let mut declared = evidence::declared();
    declared.options.details = Some(requirement_details(manifest, dependency));

    let mut analyzed = evidence::static_analysis();
    analyzed.options.details = Some(requirement_details(manifest, dependency));

    vec![declared, analyzed]
}

/// Structured details for requirement evidence.
///
/// Details carry the manifest path, group, original requirement, and PURL so the
/// graph edge stays simple while preserving enough context for UI display,
/// debugging, and future resolver passes.
fn requirement_details(manifest: &EnvironmentManifest, dependency: &Dependency) -> Object {
    let mut details = Object::new();
    details.insert(
        "ecosystem".to_string(),
        string_primitive(manifest.ecosystem.slug()),
    );
    details.insert(
        "manifest".to_string(),
        string_primitive(manifest.rel.as_str()),
    );
    details.insert("group".to_string(), string_primitive(&dependency.group));
    details.insert("name".to_string(), string_primitive(&dependency.name));
    details.insert(
        "purl".to_string(),
        string_primitive(package_purl(manifest.ecosystem, dependency)),
    );

    if let Some(raw) = &dependency.raw {
        details.insert("raw".to_string(), string_primitive(raw));
    }
    if let Some(specifier) = &dependency.specifier {
        details.insert("specifier".to_string(), string_primitive(specifier));
    }
    if !dependency.extras.is_empty() {
        details.insert(
            "extras".to_string(),
            string_primitive(dependency.extras.join(",")),
        );
    }
    if let Some(marker) = &dependency.marker {
        details.insert("marker".to_string(), string_primitive(marker));
    }

    details
}

/// Create an identifier property value.
///
/// Schema identifiers can be plain strings or `PropertyValue`s. Environment
/// graph nodes use `PropertyValue` when the identifier kind matters, such as
/// distinguishing PURLs from SHA-256 digests.
fn property_value_identifier(
    property_id: impl Into<String>,
    value: impl Into<String>,
) -> PropertyValueOrString {
    let mut identifier = PropertyValue::new(Primitive::String(value.into()));
    identifier.property_id = Some(property_id.into());
    PropertyValueOrString::PropertyValue(identifier)
}

/// Create a string primitive.
///
/// Evidence details use Schema primitives rather than ad hoc JSON values, so
/// this helper keeps map construction concise and type-correct.
fn string_primitive(value: impl Into<String>) -> Primitive {
    Primitive::String(value.into())
}

/// Compute a SHA-256 digest in `sha256:<hex>` form.
///
/// The explicit prefix makes the digest usable as an identifier value without
/// requiring consumers to infer the hash algorithm from field context.
fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    let digest = Sha256::digest(&bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut hex, "{byte:02x}");
    }

    Ok(format!("sha256:{hex}"))
}
