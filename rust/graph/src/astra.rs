//! Native projection of ASTRA contracts into workspace graphs.
//!
//! ASTRA manifests are declarations. This module parses and validates their
//! contract structure and conservatively links direct script commands, but
//! never invokes recipes.
//!
//! Stencila also recognizes the experimental `Output.target` extension
//! documented in `rust/graph/docs/astra-extensions.md`.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    ops::Range,
    path::{Component, Path, PathBuf},
};

use eyre::{Result, WrapErr, bail};
use serde::Deserialize;
use stencila_schema::{
    Array, Block, Claim, CodeLocation, CreativeWork, CreativeWorkType, Evidence, Function,
    GraphEdgeKind, GraphEvidence, Inline, Node, Object, Paragraph, Primitive,
    PropertyValueOrString, SoftwareApplication, StringOrNumber, Text, Variable,
};

use crate::{
    GraphBuilder, evidence,
    ids::{LocalGraphId, WorkspaceRelPath},
    reference::{bare_doi, doi_url, has_non_local_uri_scheme, has_remote_uri_scheme},
};

/// A parsed ASTRA manifest, retained with its source for evidence locations.
#[derive(Debug, Clone)]
struct Manifest {
    rel: WorkspaceRelPath,
    text: String,
    analysis: Analysis,
}

/// A manifest that failed YAML deserialization.
#[derive(Debug, Clone)]
struct ManifestError {
    message: String,
}

/// The supported ASTRA analysis fields.
///
/// Unknown analysis-level fields are intentionally ignored so newer ASTRA
/// metadata does not cause an otherwise projectable analysis to disappear.
/// Nested contract structures remain strict because unknown fields there may
/// change graph relationships or execution semantics.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct Analysis {
    id: Option<String>,
    version: Option<serde_yaml::Value>,
    name: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    container: Option<String>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    decisions: BTreeMap<String, Decision>,
    prior_insights: BTreeMap<String, Insight>,
    findings: BTreeMap<String, Insight>,
    analyses: BTreeMap<String, Analysis>,
    path: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Input {
    id: String,
    label: Option<String>,
    r#type: Option<String>,
    description: Option<String>,
    source: Option<String>,
    r#ref: Option<String>,
    ref_version: Option<String>,
    use_outputs: Vec<String>,
    from: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Output {
    id: String,
    label: Option<String>,
    r#type: Option<String>,
    description: Option<String>,
    /// Experimental Stencila extension: URI or path where the output is materialized.
    target: Option<String>,
    from: Option<String>,
    when: Vec<String>,
    inputs: Vec<String>,
    decisions: Vec<String>,
    recipe: Option<Recipe>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Recipe {
    command: Option<String>,
    resources: Option<Resources>,
    container: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Resources {
    cpus: Option<f64>,
    memory: Option<String>,
    time_limit: Option<String>,
    disk: Option<String>,
    gpus: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Decision {
    label: Option<String>,
    rationale: Option<String>,
    tags: Vec<String>,
    when: Vec<String>,
    from: Option<String>,
    default: Option<String>,
    options: BTreeMap<String, OptionSpec>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct OptionSpec {
    label: Option<String>,
    description: Option<String>,
    insights: Vec<String>,
    incompatible_with: Vec<String>,
    requires: Vec<String>,
    excluded: Option<bool>,
    excluded_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Insight {
    label: Option<String>,
    claim: String,
    created_at: String,
    evidence: Vec<EvidenceSpec>,
    derived: Option<bool>,
    scope: Option<String>,
    tags: Vec<String>,
    notes: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct EvidenceSpec {
    id: String,
    doi: Option<String>,
    artifact: Option<String>,
    version: Option<u64>,
    snapshot: Option<String>,
    source_commit: Option<String>,
    quote: Option<TextQuoteSelector>,
    location: Option<FragmentSelector>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct TextQuoteSelector {
    exact: String,
    prefix: Option<String>,
    suffix: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FragmentSelector {
    value: Option<String>,
    page: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Universe {
    id: String,
    description: Option<String>,
    decisions: BTreeMap<String, String>,
    analyses: BTreeMap<String, UniverseNode>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct UniverseNode {
    universe: Option<String>,
    decisions: BTreeMap<String, String>,
    analyses: BTreeMap<String, UniverseNode>,
}

#[derive(Debug, Clone)]
struct LoadedUniverse {
    rel: WorkspaceRelPath,
    scope: String,
    universe: Universe,
    selections: Vec<UniverseSelection>,
    nested: Vec<LoadedUniverse>,
}

#[derive(Debug, Clone)]
struct UniverseSelection {
    rel: WorkspaceRelPath,
    universe_id: String,
    scope: String,
    decision_scope: String,
    decision_id: String,
    option_id: String,
}

/// One analysis node after inline and path-based children have been loaded.
#[derive(Debug, Clone)]
struct AnalysisNode {
    manifest_rel: WorkspaceRelPath,
    manifest_text: String,
    source_range: Range<usize>,
    scope: Vec<String>,
    analysis: Analysis,
    children: BTreeMap<String, AnalysisNode>,
}

/// One concrete endpoint supplied by an ASTRA input.
#[derive(Debug, Clone)]
struct InputEndpoint {
    id: String,
    is_remote: bool,
}

/// Build the provisional neutral envelope used for ASTRA structural concepts.
///
/// Graph ids and edge meanings are deliberately independent of this payload so
/// purpose-built Schema types can replace these objects without graph churn.
fn astra_object(astra_type: &str, id: &str, name: &str, scope: &str) -> Object {
    Object::from([
        ("astraType", Primitive::String(astra_type.to_string())),
        ("id", Primitive::String(id.to_string())),
        ("name", Primitive::String(name.to_string())),
        ("scope", Primitive::String(scope.to_string())),
    ])
}

fn insert_string(object: &mut Object, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        object.insert(key.to_string(), Primitive::String(value.to_string()));
    }
}

fn insert_strings(object: &mut Object, key: &str, values: &[String]) {
    if !values.is_empty() {
        object.insert(
            key.to_string(),
            Primitive::Array(Array(
                values.iter().cloned().map(Primitive::String).collect(),
            )),
        );
    }
}

fn resources_object(resources: &Resources) -> Object {
    let mut object = Object::new();
    if let Some(cpus) = resources.cpus {
        object.insert("cpus".to_string(), Primitive::Number(cpus));
    }
    insert_string(&mut object, "memory", resources.memory.as_deref());
    insert_string(&mut object, "timeLimit", resources.time_limit.as_deref());
    insert_string(&mut object, "disk", resources.disk.as_deref());
    if let Some(gpus) = resources.gpus {
        object.insert("gpus".to_string(), Primitive::UnsignedInteger(gpus));
    }
    object
}

impl AnalysisNode {
    fn scope_string(&self) -> String {
        self.scope.join(".")
    }
}

/// Add every valid, unreferenced ASTRA analysis root to a workspace graph.
///
/// In permissive mode an invalid root is skipped in full, while its manifest
/// files remain present through the workspace inventory collector.
pub(crate) fn add_astra_from_workspace(
    builder: &mut GraphBuilder,
    root: &Path,
    manifest_rels: &[WorkspaceRelPath],
    resource_id: impl Fn(&WorkspaceRelPath) -> Option<String> + Copy,
    fail_on_error: bool,
) -> Result<()> {
    let mut manifests = BTreeMap::new();
    let mut parse_errors = BTreeMap::new();

    for rel in manifest_rels {
        let path = root.join(rel.as_str());
        let text = fs::read_to_string(&path)
            .wrap_err_with(|| format!("unable to read ASTRA manifest {}", rel.as_str()))?;
        match serde_yaml::from_str::<Analysis>(&text) {
            Ok(analysis) => {
                manifests.insert(
                    rel.as_str().to_string(),
                    Manifest {
                        rel: rel.clone(),
                        text,
                        analysis,
                    },
                );
            }
            Err(error) => {
                parse_errors.insert(
                    rel.as_str().to_string(),
                    ManifestError {
                        message: error.to_string(),
                    },
                );
            }
        }
    }

    let mut referenced = BTreeSet::new();
    for manifest in manifests.values() {
        collect_external_child_references(root, &manifest.rel, &manifest.analysis, &mut referenced);
    }
    if let Err(error) = detect_manifest_path_cycles(root, &manifests)
        && fail_on_error
    {
        return Err(error);
    }

    let roots = manifest_rels
        .iter()
        .filter(|rel| !referenced.contains(rel.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    for rel in roots {
        let result = (|| {
            let manifest = manifests.get(rel.as_str()).ok_or_else(|| {
                let error = parse_errors.get(rel.as_str());
                astra_error(
                    &rel,
                    "$",
                    error
                        .map(|error| error.message.as_str())
                        .unwrap_or("manifest was not parsed"),
                )
            })?;
            let mut stack = Vec::new();
            let node = load_analysis_tree(
                root,
                manifest,
                0..manifest.text.len(),
                vec!["root".to_string()],
                &manifests,
                &parse_errors,
                &mut stack,
                true,
            )?;
            validate_tree(&node)?;
            let universes = load_universes(root, &node)?;
            project_tree(builder, root, &node, &universes, resource_id)
        })();

        if let Err(error) = result
            && fail_on_error
        {
            return Err(error);
        }
    }

    Ok(())
}

/// Detect external-manifest cycles even when every manifest in the cycle is
/// referenced and therefore none would otherwise be selected as a root.
fn detect_manifest_path_cycles(root: &Path, manifests: &BTreeMap<String, Manifest>) -> Result<()> {
    fn visit(
        root: &Path,
        key: &str,
        manifests: &BTreeMap<String, Manifest>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) -> Result<()> {
        if visited.contains(key) {
            return Ok(());
        }
        let manifest = manifests
            .get(key)
            .ok_or_else(|| eyre::eyre!("ASTRA manifest `{key}` disappeared"))?;
        if !visiting.insert(key.to_string()) {
            return Err(astra_error(
                &manifest.rel,
                "analyses",
                "recursive path-based analysis reference",
            ));
        }
        let mut references = BTreeSet::new();
        collect_external_child_references(root, &manifest.rel, &manifest.analysis, &mut references);
        for child in references {
            if manifests.contains_key(&child) {
                visit(root, &child, manifests, visiting, visited)?;
            }
        }
        visiting.remove(key);
        visited.insert(key.to_string());
        Ok(())
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for key in manifests.keys() {
        visit(root, key, manifests, &mut visiting, &mut visited)?;
    }
    Ok(())
}

/// Collect path-based child manifests to distinguish children from roots.
fn collect_external_child_references(
    root: &Path,
    manifest_rel: &WorkspaceRelPath,
    analysis: &Analysis,
    referenced: &mut BTreeSet<String>,
) {
    for child in analysis.analyses.values() {
        if let Some(path) = &child.path
            && let Ok(rel) = external_child_rel(root, manifest_rel, path)
        {
            referenced.insert(rel.as_str().to_string());
        }
        collect_external_child_references(root, manifest_rel, child, referenced);
    }
}

/// Load an ASTRA analysis tree while detecting recursive external paths.
#[allow(clippy::too_many_arguments)]
fn load_analysis_tree(
    root: &Path,
    manifest: &Manifest,
    source_range: Range<usize>,
    scope: Vec<String>,
    manifests: &BTreeMap<String, Manifest>,
    parse_errors: &BTreeMap<String, ManifestError>,
    stack: &mut Vec<String>,
    track_manifest: bool,
) -> Result<AnalysisNode> {
    if track_manifest && stack.contains(&manifest.rel.as_str().to_string()) {
        return Err(astra_error(
            &manifest.rel,
            "analyses",
            "recursive path-based analysis reference",
        ));
    }
    if track_manifest {
        stack.push(manifest.rel.as_str().to_string());
    }

    validate_version(&manifest.rel, &manifest.analysis.version)?;

    let mut children = BTreeMap::new();
    for (id, child) in &manifest.analysis.analyses {
        let mut child_scope = scope.clone();
        child_scope.push(id.clone());
        let loaded = if let Some(path) = &child.path {
            if child_has_inline_content(child) {
                return Err(astra_error(
                    &manifest.rel,
                    &format!("analyses.{id}.path"),
                    "`path` is mutually exclusive with inline analysis content",
                ));
            }
            let child_rel = external_child_rel(root, &manifest.rel, path).map_err(|error| {
                astra_error(
                    &manifest.rel,
                    &format!("analyses.{id}.path"),
                    &error.to_string(),
                )
            })?;
            let child_manifest = manifests.get(child_rel.as_str()).ok_or_else(|| {
                let message = parse_errors
                    .get(child_rel.as_str())
                    .map(|error| format!("invalid child manifest: {}", error.message))
                    .unwrap_or_else(|| format!("missing child manifest {}", child_rel.as_str()));
                astra_error(&manifest.rel, &format!("analyses.{id}.path"), &message)
            })?;
            load_analysis_tree(
                root,
                child_manifest,
                0..child_manifest.text.len(),
                child_scope,
                manifests,
                parse_errors,
                stack,
                true,
            )?
        } else {
            validate_version(&manifest.rel, &child.version)?;
            let child_range = inline_analysis_range(&manifest.text, source_range.clone(), id)
                .unwrap_or_else(|| source_range.clone());
            load_inline_tree(
                root,
                &manifest.rel,
                &manifest.text,
                child_range,
                child,
                child_scope,
                manifests,
                parse_errors,
                stack,
            )?
        };
        children.insert(id.clone(), loaded);
    }

    if track_manifest {
        let _ = stack.pop();
    }
    Ok(AnalysisNode {
        manifest_rel: manifest.rel.clone(),
        manifest_text: manifest.text.clone(),
        source_range,
        scope,
        analysis: manifest.analysis.clone(),
        children,
    })
}

#[allow(clippy::too_many_arguments)]
fn load_inline_tree(
    root: &Path,
    manifest_rel: &WorkspaceRelPath,
    manifest_text: &str,
    source_range: Range<usize>,
    analysis: &Analysis,
    scope: Vec<String>,
    manifests: &BTreeMap<String, Manifest>,
    parse_errors: &BTreeMap<String, ManifestError>,
    stack: &mut Vec<String>,
) -> Result<AnalysisNode> {
    let synthetic = Manifest {
        rel: manifest_rel.clone(),
        text: manifest_text.to_string(),
        analysis: analysis.clone(),
    };
    load_analysis_tree(
        root,
        &synthetic,
        source_range,
        scope,
        manifests,
        parse_errors,
        stack,
        false,
    )
}

fn child_has_inline_content(analysis: &Analysis) -> bool {
    analysis.id.is_some()
        || analysis.version.is_some()
        || analysis.name.is_some()
        || analysis.description.is_some()
        || !analysis.tags.is_empty()
        || analysis.container.is_some()
        || !analysis.inputs.is_empty()
        || !analysis.outputs.is_empty()
        || !analysis.decisions.is_empty()
        || !analysis.prior_insights.is_empty()
        || !analysis.findings.is_empty()
        || !analysis.analyses.is_empty()
}

/// Locate one inline child analysis within its parent's source range.
fn inline_analysis_range(
    manifest_text: &str,
    parent_range: Range<usize>,
    child_id: &str,
) -> Option<Range<usize>> {
    let source = manifest_text.get(parent_range.clone())?;
    let parent_indent = if parent_range.start == 0 {
        0
    } else {
        source
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(line_indent)?
            + 2
    };
    let child_indent = parent_indent + 2;
    let mut offset = parent_range.start;
    let mut in_analyses = false;
    let mut child_start = None;

    for line in source.split_inclusive('\n') {
        let trimmed = line.trim();
        let indent = line_indent(line);
        if child_start.is_none() {
            if indent == parent_indent && trimmed == "analyses:" {
                in_analyses = true;
            } else if in_analyses && indent == child_indent && trimmed == format!("{child_id}:") {
                child_start = Some(offset);
            } else if in_analyses && !trimmed.is_empty() && indent <= parent_indent {
                return None;
            }
        } else if !trimmed.is_empty() && indent <= child_indent {
            return Some(child_start?..offset);
        }
        offset += line.len();
    }

    child_start.map(|start| start..parent_range.end)
}

fn line_indent(line: &str) -> usize {
    line.len() - line.trim_start_matches([' ', '\t']).len()
}

/// Resolve a path-based child to a conventional `astra.yaml` inside the workspace.
fn external_child_rel(
    root: &Path,
    manifest_rel: &WorkspaceRelPath,
    child_path: &str,
) -> Result<WorkspaceRelPath> {
    let child_path = Path::new(child_path);
    if child_path.is_absolute() {
        bail!("external child path must be relative to the workspace");
    }

    let parent = Path::new(manifest_rel.as_str())
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let joined = normalize_relative(parent.join(child_path))?;
    let candidate = if joined.file_name().is_some_and(|name| name == "astra.yaml") {
        joined
    } else {
        joined.join("astra.yaml")
    };
    let rel = WorkspaceRelPath::from_relative_path(&candidate)?;

    // The lexical check above rejects `..` escapes. This canonical check also
    // rejects symlink-based escapes when the target exists.
    let absolute = root.join(rel.as_str());
    if absolute.exists() {
        let canonical = absolute
            .canonicalize()
            .wrap_err_with(|| format!("unable to resolve {}", absolute.display()))?;
        if !canonical.starts_with(root) {
            bail!("external child path resolves outside the workspace");
        }
    }

    Ok(rel)
}

fn normalize_relative(path: PathBuf) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    bail!("external child path escapes the workspace");
                }
            }
            Component::RootDir | Component::Prefix(..) => {
                bail!("external child path must be workspace-relative")
            }
        }
    }
    Ok(normalized)
}

fn validate_version(rel: &WorkspaceRelPath, version: &Option<serde_yaml::Value>) -> Result<()> {
    let Some(version) = version else {
        return Ok(());
    };
    let version = yaml_scalar_string(version)
        .ok_or_else(|| astra_error(rel, "version", "version must be a scalar ASTRA version"))?;
    let mut parts = version.split('.');
    let major = parts.next().unwrap_or_default();
    let minor = parts.next();
    let patch = parts.next();
    let valid = major == "1"
        && minor == Some("0")
        && patch.is_none_or(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
        && parts.next().is_none();
    if !valid {
        return Err(astra_error(
            rel,
            "version",
            &format!("unsupported ASTRA version `{version}`; expected 1.0 or 1.0.x"),
        ));
    }
    Ok(())
}

fn yaml_scalar_string(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::String(value) => Some(value.clone()),
        serde_yaml::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

/// Validate identifiers, aliases, references, and output dependency cycles.
fn validate_tree(root: &AnalysisNode) -> Result<()> {
    let mut scopes = BTreeMap::new();
    collect_scopes(root, &mut scopes);
    for node in scopes.values() {
        validate_analysis(node, &scopes)?;
    }
    Ok(())
}

fn collect_scopes<'a>(node: &'a AnalysisNode, scopes: &mut BTreeMap<String, &'a AnalysisNode>) {
    scopes.insert(node.scope_string(), node);
    for child in node.children.values() {
        collect_scopes(child, scopes);
    }
}

fn validate_analysis(node: &AnalysisNode, scopes: &BTreeMap<String, &AnalysisNode>) -> Result<()> {
    let scope = node.scope_string();
    let mut artifacts = BTreeSet::new();
    for (index, input) in node.analysis.inputs.iter().enumerate() {
        validate_id(
            &node.manifest_rel,
            &format!("inputs[{index}].id"),
            &input.id,
        )?;
        if !artifacts.insert(input.id.as_str()) {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("inputs[{index}].id"),
                &format!("duplicate ASTRA artifact id `{}`", input.id),
            ));
        }
        if let Some(from) = &input.from {
            if input.label.is_some()
                || input.r#type.is_some()
                || input.description.is_some()
                || input.source.is_some()
                || input.r#ref.is_some()
                || input.ref_version.is_some()
                || !input.use_outputs.is_empty()
            {
                return Err(astra_error(
                    &node.manifest_rel,
                    &format!("inputs[{index}].from"),
                    "an input alias may declare only `id` and `from`",
                ));
            }
            resolve_input_from(node, from, scopes).map_err(|message| {
                astra_error(
                    &node.manifest_rel,
                    &format!("inputs[{index}].from"),
                    &message,
                )
            })?;
        } else {
            if !matches!(input.r#type.as_deref(), Some("data" | "analysis")) {
                return Err(astra_error(
                    &node.manifest_rel,
                    &format!("inputs[{index}].type"),
                    "input type must be `data` or `analysis` when `from` is absent",
                ));
            }
            if input.r#type.as_deref() == Some("analysis")
                && !input.use_outputs.is_empty()
                && input.r#ref.is_none()
            {
                return Err(astra_error(
                    &node.manifest_rel,
                    &format!("inputs[{index}].use_outputs"),
                    "`use_outputs` requires an analysis `ref`",
                ));
            }
        }
    }
    for (index, output) in node.analysis.outputs.iter().enumerate() {
        validate_id(
            &node.manifest_rel,
            &format!("outputs[{index}].id"),
            &output.id,
        )?;
        if !artifacts.insert(output.id.as_str()) {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("outputs[{index}].id"),
                &format!("duplicate ASTRA artifact id `{}`", output.id),
            ));
        }
        if let Some(from) = &output.from {
            if output.label.is_some()
                || output.r#type.is_some()
                || output.description.is_some()
                || output.target.is_some()
                || !output.inputs.is_empty()
                || !output.decisions.is_empty()
                || output.recipe.is_some()
            {
                return Err(astra_error(
                    &node.manifest_rel,
                    &format!("outputs[{index}].from"),
                    "an output re-export may declare only `id`, `from`, and `when`; its target is inherited",
                ));
            }
            resolve_output_from(node, from, scopes).map_err(|message| {
                astra_error(
                    &node.manifest_rel,
                    &format!("outputs[{index}].from"),
                    &message,
                )
            })?;
        } else {
            if let Some(target) = output.target.as_deref() {
                validate_output_target(&node.manifest_rel, index, target)?;
            }
            if !matches!(
                output.r#type.as_deref(),
                Some("metric" | "figure" | "table" | "data" | "report")
            ) {
                return Err(astra_error(
                    &node.manifest_rel,
                    &format!("outputs[{index}].type"),
                    "output type must be metric, figure, table, data, or report",
                ));
            }
            if let Some(command) = output
                .recipe
                .as_ref()
                .and_then(|recipe| recipe.command.as_deref())
            {
                validate_recipe_command(command, output).map_err(|message| {
                    astra_error(
                        &node.manifest_rel,
                        &format!("outputs[{index}].recipe.command"),
                        &message,
                    )
                })?;
            }
            for input in &output.inputs {
                if !node.analysis.inputs.iter().any(|item| item.id == *input)
                    && resolve_output_dependency(node, input, scopes).is_err()
                {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("outputs[{index}].inputs"),
                        &format!("unresolved input or sibling output `{input}` in `{scope}`"),
                    ));
                }
            }
            if let Some(resources) = output
                .recipe
                .as_ref()
                .and_then(|recipe| recipe.resources.as_ref())
            {
                if resources
                    .cpus
                    .is_some_and(|value| !value.is_finite() || value < 0.0)
                {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("outputs[{index}].recipe.resources.cpus"),
                        "recipe CPUs must be finite and non-negative",
                    ));
                }
                if resources.gpus == Some(0) {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("outputs[{index}].recipe.resources.gpus"),
                        "recipe GPU count must be greater than zero",
                    ));
                }
                for (name, value) in [
                    ("memory", resources.memory.as_deref()),
                    ("time_limit", resources.time_limit.as_deref()),
                    ("disk", resources.disk.as_deref()),
                ] {
                    if value.is_some_and(str::is_empty) {
                        return Err(astra_error(
                            &node.manifest_rel,
                            &format!("outputs[{index}].recipe.resources.{name}"),
                            "resource values must not be empty",
                        ));
                    }
                }
            }
            for decision in &output.decisions {
                if !node.analysis.decisions.contains_key(decision) {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("outputs[{index}].decisions"),
                        &format!("unresolved decision `{decision}` in `{scope}`"),
                    ));
                }
            }
        }
        validate_conditions(node, &output.when, scopes).map_err(|message| {
            astra_error(
                &node.manifest_rel,
                &format!("outputs[{index}].when"),
                &message,
            )
        })?;
    }
    for (id, decision) in &node.analysis.decisions {
        validate_id(&node.manifest_rel, &format!("decisions.{id}"), id)?;
        if let Some(from) = &decision.from {
            if decision.label.is_some()
                || decision.rationale.is_some()
                || !decision.tags.is_empty()
                || decision.default.is_some()
                || !decision.options.is_empty()
            {
                return Err(astra_error(
                    &node.manifest_rel,
                    &format!("decisions.{id}.from"),
                    "a decision alias may declare only `from` and `when`",
                ));
            }
            resolve_decision_from(node, from, scopes).map_err(|message| {
                astra_error(
                    &node.manifest_rel,
                    &format!("decisions.{id}.from"),
                    &message,
                )
            })?;
        } else if decision.label.is_none() || decision.options.is_empty() {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("decisions.{id}"),
                "a decision requires `label` and at least one option",
            ));
        }
        if let Some(default) = &decision.default
            && !decision.options.contains_key(default)
        {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("decisions.{id}.default"),
                &format!("default option `{default}` is not declared"),
            ));
        }
        if decision.from.is_none() {
            for (option_id, option) in &decision.options {
                validate_id(
                    &node.manifest_rel,
                    &format!("decisions.{id}.options.{option_id}"),
                    option_id,
                )?;
                if option.label.is_none() {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("decisions.{id}.options.{option_id}.label"),
                        "an option requires `label`",
                    ));
                }
                if option.excluded_reason.is_some() && option.excluded != Some(true) {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("decisions.{id}.options.{option_id}.excluded_reason"),
                        "excluded_reason requires excluded to be true",
                    ));
                }
                if option.excluded == Some(true) && option.excluded_reason.is_none() {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("decisions.{id}.options.{option_id}.excluded"),
                        "an excluded option requires an excluded_reason",
                    ));
                }
                if decision.default.as_deref() == Some(option_id.as_str())
                    && option.excluded == Some(true)
                {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("decisions.{id}.default"),
                        "the default option must not be excluded",
                    ));
                }
                for insight in &option.insights {
                    resolve_prior_insight(node, insight, scopes).map_err(|message| {
                        astra_error(
                            &node.manifest_rel,
                            &format!("decisions.{id}.options.{option_id}.insights"),
                            &message,
                        )
                    })?;
                }
                for constraint in option
                    .requires
                    .iter()
                    .chain(option.incompatible_with.iter())
                {
                    validate_constraint(node, constraint, scopes).map_err(|message| {
                        astra_error(
                            &node.manifest_rel,
                            &format!("decisions.{id}.options.{option_id}"),
                            &message,
                        )
                    })?;
                }
            }
        }
        validate_conditions(node, &decision.when, scopes).map_err(|message| {
            astra_error(
                &node.manifest_rel,
                &format!("decisions.{id}.when"),
                &message,
            )
        })?;
        if decision
            .when
            .iter()
            .filter_map(|condition| condition_target(condition).ok())
            .any(|(decision_id, _)| decision_id == id)
        {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("decisions.{id}.when"),
                "a decision condition must not reference itself",
            ));
        }
    }
    for (collection, insights) in [
        ("prior_insights", &node.analysis.prior_insights),
        ("findings", &node.analysis.findings),
    ] {
        for (id, insight) in insights {
            validate_insight(node, scopes, collection, id, insight)?;
        }
    }
    validate_output_cycles(node)
}

fn validate_constraint(
    node: &AnalysisNode,
    constraint: &str,
    scopes: &BTreeMap<String, &AnalysisNode>,
) -> std::result::Result<(), String> {
    let (decision, option) = condition_target(constraint)?;
    let decision = effective_decision(node, decision, scopes)?;
    if decision.options.contains_key(option) {
        Ok(())
    } else {
        Err(format!(
            "constraint '{constraint}' references an undeclared option"
        ))
    }
}

fn validate_insight(
    node: &AnalysisNode,
    scopes: &BTreeMap<String, &AnalysisNode>,
    collection: &str,
    id: &str,
    insight: &Insight,
) -> Result<()> {
    let field = format!("{collection}.{id}");
    validate_id(&node.manifest_rel, &field, id)?;
    if insight.claim.trim().is_empty()
        || insight.created_at.trim().is_empty()
        || insight.evidence.is_empty()
    {
        return Err(astra_error(
            &node.manifest_rel,
            &field,
            "an insight requires claim, created_at, and evidence",
        ));
    }
    chrono::DateTime::parse_from_rfc3339(&insight.created_at).map_err(|_| {
        astra_error(
            &node.manifest_rel,
            &format!("{field}.created_at"),
            "insight timestamp must be ISO 8601 with a timezone",
        )
    })?;
    let mut evidence_ids = BTreeSet::new();
    for evidence in &insight.evidence {
        validate_id(
            &node.manifest_rel,
            &format!("{field}.evidence.id"),
            &evidence.id,
        )?;
        if !evidence_ids.insert(&evidence.id) {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("{field}.evidence"),
                &format!("duplicate evidence id '{}'", evidence.id),
            ));
        }
        if evidence.doi.is_some() == evidence.artifact.is_some() {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("{field}.evidence.{}", evidence.id),
                "evidence requires exactly one of doi or artifact",
            ));
        }
        if let Some(doi) = &evidence.doi
            && !is_valid_astra_doi(doi)
        {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("{field}.evidence.{}.doi", evidence.id),
                "invalid DOI",
            ));
        }
        if evidence.version == Some(0)
            || evidence
                .location
                .as_ref()
                .and_then(|location| location.page)
                == Some(0)
        {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("{field}.evidence.{}", evidence.id),
                "evidence versions and pages are 1-indexed",
            ));
        }
        if evidence
            .quote
            .as_ref()
            .is_some_and(|quote| quote.exact.trim().is_empty())
        {
            return Err(astra_error(
                &node.manifest_rel,
                &format!("{field}.evidence.{}.quote.exact", evidence.id),
                "evidence quote must not be empty",
            ));
        }
        if let Some(artifact) = &evidence.artifact {
            resolve_output_dependency(node, artifact, scopes).map_err(|message| {
                astra_error(
                    &node.manifest_rel,
                    &format!("{field}.evidence.{}.artifact", evidence.id),
                    &message,
                )
            })?;
        }
    }
    Ok(())
}

fn is_valid_astra_doi(value: &str) -> bool {
    bare_doi(value).is_some_and(|doi| {
        doi.strip_prefix("10.")
            .and_then(|rest| rest.split_once('/'))
            .is_some_and(|(registrant, suffix)| {
                registrant.len() >= 4
                    && registrant
                        .chars()
                        .all(|character| character.is_ascii_digit())
                    && !suffix.is_empty()
            })
    })
}

/// Validate typed placeholders in one ASTRA recipe command.
fn validate_recipe_command(command: &str, output: &Output) -> std::result::Result<(), String> {
    let bytes = command.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'{' if bytes.get(index + 1) == Some(&b'{') => index += 2,
            b'{' if index > 0 && bytes[index - 1] == b'$' => {
                let Some(end) = command[index + 1..].find('}') else {
                    return Err("unterminated shell variable expansion".to_string());
                };
                index += end + 2;
            }
            b'{' => {
                let Some(end) = command[index + 1..].find('}') else {
                    return Err("unterminated recipe placeholder".to_string());
                };
                let end = index + 1 + end;
                let placeholder = &command[index + 1..end];
                match placeholder {
                    "inputs" | "output" => {}
                    _ if placeholder
                        .strip_prefix("inputs.")
                        .is_some_and(|id| output.inputs.iter().any(|input| input == id)) => {}
                    _ if placeholder.strip_prefix("decisions.").is_some_and(|id| {
                        output.decisions.iter().any(|decision| decision == id)
                    }) => {}
                    _ => {
                        return Err(format!(
                            "recipe placeholder `{{{placeholder}}}` is not declared by the output"
                        ));
                    }
                }
                index = end + 1;
            }
            b'}' if bytes.get(index + 1) == Some(&b'}') => index += 2,
            b'}' => return Err("unmatched `}` in recipe command".to_string()),
            _ => index += 1,
        }
    }
    Ok(())
}

/// Validate the experimental ASTRA `Output.target` extension.
fn validate_output_target(
    manifest_rel: &WorkspaceRelPath,
    output_index: usize,
    target: &str,
) -> Result<()> {
    let field = format!("outputs[{output_index}].target");
    if target.trim().is_empty() {
        return Err(astra_error(
            manifest_rel,
            &field,
            "output target must not be empty",
        ));
    }

    if !has_non_local_uri_scheme(target) {
        let parent = Path::new(manifest_rel.as_str())
            .parent()
            .unwrap_or_else(|| Path::new(""));
        normalize_relative(parent.join(target)).map_err(|error| {
            astra_error(
                manifest_rel,
                &field,
                &format!("output target must stay within the workspace: {error}"),
            )
        })?;
    }

    Ok(())
}

/// Validate decision-option conditions within one analysis scope.
fn validate_conditions(
    node: &AnalysisNode,
    conditions: &[String],
    scopes: &BTreeMap<String, &AnalysisNode>,
) -> std::result::Result<(), String> {
    for condition in conditions {
        let (decision_id, option_id) = condition_target(condition)?;
        let decision = effective_decision(node, decision_id, scopes)?;
        if !decision.options.contains_key(option_id) {
            return Err(format!(
                "condition `{condition}` references undeclared option `{option_id}`"
            ));
        }
    }
    Ok(())
}

fn condition_target(condition: &str) -> std::result::Result<(&str, &str), String> {
    let condition = condition.strip_prefix('~').unwrap_or(condition);
    let Some((decision, option)) = condition.split_once('.') else {
        return Err(format!(
            "condition `{condition}` must use `decision.option` syntax"
        ));
    };
    if decision.is_empty() || option.is_empty() || option.contains('.') {
        return Err(format!(
            "condition `{condition}` must use `decision.option` syntax"
        ));
    }
    Ok((decision, option))
}

fn effective_decision<'a>(
    node: &'a AnalysisNode,
    id: &str,
    scopes: &BTreeMap<String, &'a AnalysisNode>,
) -> std::result::Result<&'a Decision, String> {
    effective_decision_inner(node, id, scopes, &mut BTreeSet::new())
}

fn effective_decision_inner<'a>(
    node: &'a AnalysisNode,
    id: &str,
    scopes: &BTreeMap<String, &'a AnalysisNode>,
    visiting: &mut BTreeSet<(String, String)>,
) -> std::result::Result<&'a Decision, String> {
    let key = (node.scope_string(), id.to_string());
    if !visiting.insert(key.clone()) {
        return Err(format!("decision alias cycle includes '{id}'"));
    }
    let decision = node
        .analysis
        .decisions
        .get(id)
        .ok_or_else(|| format!("condition references undeclared decision `{id}`"))?;
    let Some(from) = &decision.from else {
        return Ok(decision);
    };
    let (target_scope, target_id) = resolve_decision_from(node, from, scopes)?;
    let target = scope_node(&target_scope, scopes)?;
    let resolved = effective_decision_inner(target, &target_id, scopes, visiting)?;
    visiting.remove(&key);
    Ok(resolved)
}

fn defining_decision(
    node: &AnalysisNode,
    id: &str,
    scopes: &BTreeMap<String, &AnalysisNode>,
) -> std::result::Result<(String, String), String> {
    let decision = node
        .analysis
        .decisions
        .get(id)
        .ok_or_else(|| format!("undeclared decision '{id}'"))?;
    let Some(from) = &decision.from else {
        return Ok((node.scope_string(), id.to_string()));
    };
    let (scope, target_id) = resolve_decision_from(node, from, scopes)?;
    defining_decision(scope_node(&scope, scopes)?, &target_id, scopes)
}

/// Resolve an option insight reference to a local or ancestor prior insight.
fn resolve_prior_insight(
    node: &AnalysisNode,
    reference: &str,
    scopes: &BTreeMap<String, &AnalysisNode>,
) -> std::result::Result<(String, String), String> {
    let (scope, id) = if reference.starts_with("../") {
        let (scope, path) = ascend_from(node, reference)?;
        if path.len() != 1 {
            return Err(format!(
                "option insight '{reference}' must reference one ancestor prior insight"
            ));
        }
        (scope, path[0].clone())
    } else {
        if reference.contains(['/', '.']) {
            return Err(format!(
                "option insight '{reference}' must be a local id or an ancestor reference"
            ));
        }
        (node.scope_string(), reference.to_string())
    };
    let target = scope_node(&scope, scopes)?;
    if target.analysis.prior_insights.contains_key(&id) {
        Ok((scope, id))
    } else {
        Err(format!(
            "unresolved prior insight '{reference}' in analysis scope '{scope}'"
        ))
    }
}

fn validate_id(rel: &WorkspaceRelPath, field: &str, id: &str) -> Result<()> {
    const RESERVED: [&str; 8] = [
        "inputs",
        "outputs",
        "decisions",
        "findings",
        "prior_insights",
        "analyses",
        "options",
        "content",
    ];
    let valid = !id.is_empty()
        && !RESERVED.contains(&id)
        && id
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase())
        && id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        });
    if !valid {
        return Err(astra_error(rel, field, &format!("invalid ASTRA id `{id}`")));
    }
    Ok(())
}

fn validate_output_cycles(node: &AnalysisNode) -> Result<()> {
    let outputs = node
        .analysis
        .outputs
        .iter()
        .map(|output| (output.id.as_str(), output))
        .collect::<BTreeMap<_, _>>();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in outputs.keys() {
        visit_output(node, id, &outputs, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit_output<'a>(
    node: &AnalysisNode,
    id: &'a str,
    outputs: &BTreeMap<&'a str, &'a Output>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Result<()> {
    if visited.contains(id) {
        return Ok(());
    }
    if !visiting.insert(id) {
        return Err(astra_error(
            &node.manifest_rel,
            &format!("outputs.{id}.inputs"),
            &format!("output dependency cycle includes `{id}`"),
        ));
    }
    if let Some(output) = outputs.get(id) {
        for dependency in &output.inputs {
            if outputs.contains_key(dependency.as_str()) {
                visit_output(node, dependency, outputs, visiting, visited)?;
            }
        }
    }
    visiting.remove(id);
    visited.insert(id);
    Ok(())
}

fn resolve_input_from(
    node: &AnalysisNode,
    from: &str,
    scopes: &BTreeMap<String, &AnalysisNode>,
) -> std::result::Result<(String, String), String> {
    let (base, path) = ascend_from(node, from)?;
    if path.len() == 1 {
        let target = scope_node(&base, scopes)?;
        if target
            .analysis
            .inputs
            .iter()
            .any(|input| input.id == path[0])
        {
            return Ok((base, path[0].clone()));
        }
    } else if path.len() >= 2 {
        let child_scope = join_scope(&base, &path[..path.len() - 1]);
        let target = scope_node(&child_scope, scopes)?;
        let id = path.last().cloned().unwrap_or_default();
        if target.analysis.outputs.iter().any(|output| output.id == id) {
            return Ok((child_scope, id));
        }
    }
    Err(format!("unresolved or illegal Input.from `{from}`"))
}

fn resolve_output_from(
    node: &AnalysisNode,
    from: &str,
    scopes: &BTreeMap<String, &AnalysisNode>,
) -> std::result::Result<(String, String), String> {
    if from.starts_with("../") {
        return Err(format!("illegal upward Output.from `{from}`"));
    }
    let path = split_dotted(from);
    if path.len() < 2 {
        return Err(format!("Output.from must name a child output: `{from}`"));
    }
    let child_scope = join_scope(&node.scope_string(), &path[..path.len() - 1]);
    let target = scope_node(&child_scope, scopes)?;
    let id = path.last().cloned().unwrap_or_default();
    if target.analysis.outputs.iter().any(|output| output.id == id) {
        Ok((child_scope, id))
    } else {
        Err(format!("unresolved Output.from `{from}`"))
    }
}

/// Resolve a local or qualified descendant output dependency.
fn resolve_output_dependency(
    node: &AnalysisNode,
    value: &str,
    scopes: &BTreeMap<String, &AnalysisNode>,
) -> std::result::Result<(String, String), String> {
    let path = split_dotted(value);
    if path.len() == 1
        && node
            .analysis
            .outputs
            .iter()
            .any(|output| output.id == path[0])
    {
        return Ok((node.scope_string(), path[0].clone()));
    }
    if path.len() >= 2 {
        let child_scope = join_scope(&node.scope_string(), &path[..path.len() - 1]);
        let target = scope_node(&child_scope, scopes)?;
        let id = path.last().cloned().unwrap_or_default();
        if target.analysis.outputs.iter().any(|output| output.id == id) {
            return Ok((child_scope, id));
        }
    }
    Err(format!(
        "unresolved output dependency \u{0060}{value}\u{0060}"
    ))
}

/// Resolve an output re-export chain to the output that declares its metadata.
fn resolve_effective_output<'a>(
    node: &'a AnalysisNode,
    output: &'a Output,
    scopes: &BTreeMap<String, &'a AnalysisNode>,
) -> std::result::Result<(&'a AnalysisNode, &'a Output), String> {
    let Some(from) = &output.from else {
        return Ok((node, output));
    };
    let (target_scope, target_id) = resolve_output_from(node, from, scopes)?;
    let target_node = scope_node(&target_scope, scopes)?;
    let target_output = target_node
        .analysis
        .outputs
        .iter()
        .find(|output| output.id == target_id)
        .ok_or_else(|| format!("resolved Output.from `{from}` disappeared"))?;
    resolve_effective_output(target_node, target_output, scopes)
}

fn resolve_decision_from(
    node: &AnalysisNode,
    from: &str,
    scopes: &BTreeMap<String, &AnalysisNode>,
) -> std::result::Result<(String, String), String> {
    let (base, path) = ascend_from(node, from)?;
    if path.len() != 1 {
        return Err(format!("illegal Decision.from direction `{from}`"));
    }
    let target = scope_node(&base, scopes)?;
    if target.analysis.decisions.contains_key(&path[0]) {
        Ok((base, path[0].clone()))
    } else {
        Err(format!("unresolved Decision.from `{from}`"))
    }
}

fn ascend_from(
    node: &AnalysisNode,
    from: &str,
) -> std::result::Result<(String, Vec<String>), String> {
    let mut remaining = from;
    let mut scope = node.scope.clone();
    let mut ascents = 0;
    while let Some(rest) = remaining.strip_prefix("../") {
        ascents += 1;
        remaining = rest;
    }
    if ascents == 0 {
        return Err(format!("Input.from or Decision.from must ascend: `{from}`"));
    }
    for _ in 0..ascents {
        if scope.len() <= 1 {
            return Err(format!("`{from}` escapes the ASTRA analysis root"));
        }
        scope.pop();
    }
    let path = split_dotted(remaining);
    if path.is_empty() {
        return Err(format!("incomplete ASTRA path `{from}`"));
    }
    Ok((scope.join("."), path))
}

fn split_dotted(path: &str) -> Vec<String> {
    path.split('.')
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn join_scope(base: &str, path: &[String]) -> String {
    if path.is_empty() {
        base.to_string()
    } else {
        format!("{base}.{}", path.join("."))
    }
}

fn scope_node<'a>(
    scope: &str,
    scopes: &BTreeMap<String, &'a AnalysisNode>,
) -> std::result::Result<&'a AnalysisNode, String> {
    scopes
        .get(scope)
        .copied()
        .ok_or_else(|| format!("unresolved analysis scope `{scope}`"))
}

fn load_universes(workspace_root: &Path, root: &AnalysisNode) -> Result<Vec<LoadedUniverse>> {
    let parent = Path::new(root.manifest_rel.as_str())
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let directory = workspace_root.join(parent).join("universes");
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    ensure_workspace_path(workspace_root, &directory)?;
    let mut paths = fs::read_dir(&directory)
        .wrap_err_with(|| format!("unable to read {}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| matches!(extension, "yaml" | "yml"))
        })
        .collect::<Vec<_>>();
    paths.sort();

    let mut scopes = BTreeMap::new();
    collect_scopes(root, &mut scopes);
    paths
        .into_iter()
        .map(|path| {
            ensure_workspace_path(workspace_root, &path)?;
            let rel = WorkspaceRelPath::from_workspace_path(workspace_root, &path)?;
            let text = fs::read_to_string(&path)
                .wrap_err_with(|| format!("unable to read universe {}", rel.as_str()))?;
            let universe = serde_yaml::from_str::<Universe>(&text)
                .map_err(|error| astra_error(&rel, "$", &error.to_string()))?;
            validate_universe_id(&rel, &universe.id)?;
            let mut selections = Vec::new();
            let mut nested = Vec::new();
            validate_universe_node(
                workspace_root,
                root,
                &universe.decisions,
                &universe.analyses,
                Some(&rel),
                &universe.id,
                &scopes,
                &BTreeMap::new(),
                &mut vec![rel.as_str().to_string()],
                &mut selections,
                &mut nested,
            )?;
            Ok(LoadedUniverse {
                rel,
                scope: root.scope_string(),
                universe,
                selections,
                nested,
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn validate_universe_node(
    workspace_root: &Path,
    analysis: &AnalysisNode,
    decisions: &BTreeMap<String, String>,
    analyses: &BTreeMap<String, UniverseNode>,
    universe_rel: Option<&WorkspaceRelPath>,
    universe_id: &str,
    scopes: &BTreeMap<String, &AnalysisNode>,
    ancestor_selections: &BTreeMap<String, BTreeMap<String, String>>,
    stack: &mut Vec<String>,
    selections: &mut Vec<UniverseSelection>,
    nested_universes: &mut Vec<LoadedUniverse>,
) -> Result<()> {
    let error_rel = universe_rel.unwrap_or(&analysis.manifest_rel);
    for id in decisions.keys() {
        match analysis.analysis.decisions.get(id) {
            None => {
                return Err(astra_error(
                    error_rel,
                    "decisions",
                    &format!("universe selects unknown decision '{id}'"),
                ));
            }
            Some(decision) if decision.from.is_some() => {
                return Err(astra_error(
                    error_rel,
                    "decisions",
                    &format!(
                        "universe must not select inherited decision '{id}'; its value comes from an ancestor"
                    ),
                ));
            }
            Some(..) => {}
        }
    }

    let mut effective_selections = decisions.clone();
    for (id, decision) in &analysis.analysis.decisions {
        let Some(from) = &decision.from else {
            continue;
        };
        let (target_scope, target_id) = resolve_decision_from(analysis, from, scopes)
            .map_err(|message| astra_error(error_rel, "decisions", &message))?;
        if let Some(option_id) = ancestor_selections
            .get(&target_scope)
            .and_then(|selections| selections.get(&target_id))
        {
            effective_selections.insert(id.clone(), option_id.clone());
        }
    }

    for (id, decision) in analysis
        .analysis
        .decisions
        .iter()
        .filter(|(_, decision)| decision.from.is_none())
    {
        let active = decision.when.iter().all(|condition| {
            condition_target(condition).is_ok_and(|(target, option)| {
                let selected = effective_selections
                    .get(target)
                    .is_some_and(|value| value == option);
                if condition.starts_with('~') {
                    !selected
                } else {
                    selected
                }
            })
        });
        let selected = decisions.get(id);
        if active && selected.is_none() {
            return Err(astra_error(
                error_rel,
                "decisions",
                &format!("universe is missing active decision '{id}'"),
            ));
        }
        if !active && selected.is_some() {
            return Err(astra_error(
                error_rel,
                "decisions",
                &format!("universe selects inactive decision '{id}'"),
            ));
        }
        let Some(option_id) = selected else {
            continue;
        };
        let option = decision.options.get(option_id).ok_or_else(|| {
            astra_error(
                error_rel,
                "decisions",
                &format!("decision '{id}' has no option '{option_id}'"),
            )
        })?;
        if option.excluded == Some(true) {
            return Err(astra_error(
                error_rel,
                "decisions",
                &format!("universe selects excluded option '{id}.{option_id}'"),
            ));
        }
        for required in &option.requires {
            let (required_decision, required_option) = condition_target(required)
                .map_err(|message| astra_error(error_rel, "decisions", &message))?;
            if effective_selections
                .get(required_decision)
                .map(String::as_str)
                != Some(required_option)
            {
                return Err(astra_error(
                    error_rel,
                    "decisions",
                    &format!("option '{id}.{option_id}' requires '{required}'"),
                ));
            }
        }
        for incompatible in &option.incompatible_with {
            let (other_decision, other_option) = condition_target(incompatible)
                .map_err(|message| astra_error(error_rel, "decisions", &message))?;
            if effective_selections.get(other_decision).map(String::as_str) == Some(other_option) {
                return Err(astra_error(
                    error_rel,
                    "decisions",
                    &format!("option '{id}.{option_id}' is incompatible with '{incompatible}'"),
                ));
            }
        }
        selections.push(UniverseSelection {
            rel: error_rel.clone(),
            universe_id: universe_id.to_string(),
            scope: analysis.scope_string(),
            decision_scope: analysis.scope_string(),
            decision_id: id.clone(),
            option_id: option_id.clone(),
        });
    }

    let mut selections_by_scope = ancestor_selections.clone();
    selections_by_scope.insert(analysis.scope_string(), effective_selections);

    for id in analyses.keys() {
        if !analysis.children.contains_key(id) {
            return Err(astra_error(
                error_rel,
                "analyses",
                &format!("universe references unknown sub-analysis '{id}'"),
            ));
        }
    }
    for (id, child) in &analysis.children {
        let Some(node) = analyses.get(id) else {
            validate_universe_node(
                workspace_root,
                child,
                &BTreeMap::new(),
                &BTreeMap::new(),
                Some(error_rel),
                universe_id,
                scopes,
                &selections_by_scope,
                stack,
                selections,
                nested_universes,
            )?;
            continue;
        };
        if let Some(name) = &node.universe {
            if !node.decisions.is_empty() || !node.analyses.is_empty() {
                return Err(astra_error(
                    error_rel,
                    &format!("analyses.{id}"),
                    "a named child universe may not include inline selections",
                ));
            }
            let parent = Path::new(child.manifest_rel.as_str())
                .parent()
                .unwrap_or_else(|| Path::new(""));
            let path = workspace_root
                .join(parent)
                .join("universes")
                .join(format!("{name}.yaml"));
            if !path.is_file() {
                return Err(astra_error(
                    error_rel,
                    &format!("analyses.{id}.universe"),
                    &format!("missing named child universe '{name}'"),
                ));
            }
            ensure_workspace_path(workspace_root, &path)?;
            let rel = WorkspaceRelPath::from_workspace_path(workspace_root, &path)?;
            if stack.contains(&rel.as_str().to_string()) {
                return Err(astra_error(&rel, "$", "universe reference cycle"));
            }
            stack.push(rel.as_str().to_string());
            let text = fs::read_to_string(&path)?;
            let nested = serde_yaml::from_str::<Universe>(&text)
                .map_err(|error| astra_error(&rel, "$", &error.to_string()))?;
            validate_universe_id(&rel, &nested.id)?;
            let mut nested_selections = Vec::new();
            let mut nested_children = Vec::new();
            validate_universe_node(
                workspace_root,
                child,
                &nested.decisions,
                &nested.analyses,
                Some(&rel),
                &nested.id,
                scopes,
                &selections_by_scope,
                stack,
                &mut nested_selections,
                &mut nested_children,
            )?;
            selections.extend(nested_selections.iter().cloned());
            nested_universes.push(LoadedUniverse {
                rel,
                scope: child.scope_string(),
                universe: nested,
                selections: nested_selections,
                nested: nested_children,
            });
            let _ = stack.pop();
        } else {
            validate_universe_node(
                workspace_root,
                child,
                &node.decisions,
                &node.analyses,
                Some(error_rel),
                universe_id,
                scopes,
                &selections_by_scope,
                stack,
                selections,
                nested_universes,
            )?;
        }
    }
    Ok(())
}

/// Ensure a discovered or referenced ASTRA path does not traverse a symlink
/// outside the canonical workspace root.
fn ensure_workspace_path(workspace_root: &Path, path: &Path) -> Result<()> {
    let canonical = path
        .canonicalize()
        .wrap_err_with(|| format!("unable to resolve {}", path.display()))?;
    if !canonical.starts_with(workspace_root) {
        bail!(
            "ASTRA universe path resolves outside the workspace: {}",
            path.display()
        );
    }
    Ok(())
}

fn validate_universe_id(rel: &WorkspaceRelPath, id: &str) -> Result<()> {
    let valid = id
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
        && id.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '_' | '-')
        });
    if valid {
        Ok(())
    } else {
        Err(astra_error(
            rel,
            "id",
            &format!("invalid universe id '{id}'"),
        ))
    }
}

/// Project a validated ASTRA root using its scoped symbol tables.
fn project_tree(
    builder: &mut GraphBuilder,
    workspace_root: &Path,
    root: &AnalysisNode,
    universes: &[LoadedUniverse],
    resource_id: impl Fn(&WorkspaceRelPath) -> Option<String> + Copy,
) -> Result<()> {
    let root_key = root.manifest_rel.as_str();
    let mut scopes = BTreeMap::new();
    collect_scopes(root, &mut scopes);
    project_analysis(
        builder,
        workspace_root,
        root_key,
        root,
        &scopes,
        resource_id,
        None,
    )?;
    project_universes(builder, root_key, root, universes, resource_id);
    Ok(())
}

fn project_universes(
    builder: &mut GraphBuilder,
    root_key: &str,
    _root: &AnalysisNode,
    universes: &[LoadedUniverse],
    resource_id: impl Fn(&WorkspaceRelPath) -> Option<String> + Copy,
) {
    for loaded in universes {
        project_universe(builder, root_key, loaded, resource_id);
    }
}

fn project_universe(
    builder: &mut GraphBuilder,
    root_key: &str,
    loaded: &LoadedUniverse,
    resource_id: impl Fn(&WorkspaceRelPath) -> Option<String> + Copy,
) {
    let scope = &loaded.scope;
    let analysis_id = LocalGraphId::astra_analysis(root_key, scope);
    let universe_id = LocalGraphId::astra_universe(root_key, scope, &loaded.universe.id);
    let mut object = astra_object("Universe", &loaded.universe.id, &loaded.universe.id, scope);
    insert_string(
        &mut object,
        "description",
        loaded.universe.description.as_deref(),
    );
    builder.add_schema_node(&universe_id, Node::Object(object));
    let evidence = vec![universe_evidence(&loaded.rel, &loaded.universe.id)];
    builder.add_containment(&universe_id, &analysis_id, evidence.clone());
    builder.add_edge_with_evidence(
        &universe_id,
        &analysis_id,
        GraphEdgeKind::Configures,
        evidence.clone(),
    );
    if let Some(manifest_id) = resource_id(&loaded.rel) {
        builder.add_declaration(&manifest_id, &universe_id, evidence.clone());
    }
    for selection in &loaded.selections {
        let option_id = LocalGraphId::astra_option(
            root_key,
            &selection.decision_scope,
            &selection.decision_id,
            &selection.option_id,
        );
        let mut selection_evidence =
            vec![universe_evidence(&selection.rel, &selection.universe_id)];
        if let Some(item) = selection_evidence.first_mut() {
            let details = item.options.details.get_or_insert_with(Object::new);
            details.insert(
                "analysisScope".to_string(),
                Primitive::String(selection.scope.clone()),
            );
            details.insert(
                "selection".to_string(),
                Primitive::String(format!("{}.{}", selection.decision_id, selection.option_id)),
            );
        }
        builder.add_edge_with_evidence(
            option_id,
            &universe_id,
            GraphEdgeKind::Configures,
            selection_evidence,
        );
    }
    for nested in &loaded.nested {
        project_universe(builder, root_key, nested, resource_id);
    }
}

fn universe_evidence(rel: &WorkspaceRelPath, id: &str) -> GraphEvidence {
    let mut evidence = evidence::declared_at(rel.as_str(), None, None);
    evidence.code_location = Some(CodeLocation {
        source: Some(rel.as_str().to_string()),
        ..Default::default()
    });
    evidence.options.details = Some(Object::from([
        (
            "detector",
            Primitive::String("stencila-astra-contract".to_string()),
        ),
        ("fieldPath", Primitive::String("decisions".to_string())),
        ("id", Primitive::String(id.to_string())),
    ]));
    evidence
}

#[allow(clippy::too_many_arguments)]
fn project_analysis(
    builder: &mut GraphBuilder,
    workspace_root: &Path,
    root_key: &str,
    node: &AnalysisNode,
    scopes: &BTreeMap<String, &AnalysisNode>,
    resource_id: impl Fn(&WorkspaceRelPath) -> Option<String> + Copy,
    parent_id: Option<&str>,
) -> Result<()> {
    let scope = node.scope_string();
    let analysis_id = LocalGraphId::astra_analysis(root_key, &scope);
    let local_id = node
        .analysis
        .id
        .as_deref()
        .or_else(|| node.scope.last().map(String::as_str))
        .unwrap_or("root");
    let name = node
        .analysis
        .name
        .clone()
        .or_else(|| node.analysis.id.clone())
        .unwrap_or_else(|| local_id.to_string());
    let mut analysis_object = astra_object("Analysis", local_id, &name, &scope);
    insert_string(
        &mut analysis_object,
        "description",
        node.analysis.description.as_deref(),
    );
    insert_string(
        &mut analysis_object,
        "version",
        node.analysis
            .version
            .as_ref()
            .and_then(yaml_scalar_string)
            .as_deref(),
    );
    insert_strings(&mut analysis_object, "tags", &node.analysis.tags);
    insert_string(
        &mut analysis_object,
        "container",
        node.analysis.container.as_deref(),
    );
    builder.add_schema_node(&analysis_id, Node::Object(analysis_object));

    let manifest_id = resource_id(&node.manifest_rel).ok_or_else(|| {
        astra_error(
            &node.manifest_rel,
            "$",
            "ASTRA manifest has no workspace inventory node",
        )
    })?;
    builder.add_declaration(
        &manifest_id,
        &analysis_id,
        vec![declared_evidence(node, "$", None, None)],
    );
    if let Some(parent_id) = parent_id {
        builder.add_containment(
            &analysis_id,
            parent_id,
            vec![declared_evidence(node, "analyses", None, None)],
        );
    }

    let mut input_endpoints = BTreeMap::<String, Vec<InputEndpoint>>::new();
    for (index, input) in node.analysis.inputs.iter().enumerate() {
        let endpoints = project_input(
            builder,
            workspace_root,
            root_key,
            node,
            scopes,
            resource_id,
            index,
            input,
        )?;
        input_endpoints.insert(input.id.clone(), endpoints);
    }

    let mut decision_endpoints = BTreeMap::new();
    for (id, decision) in &node.analysis.decisions {
        let effective = effective_decision(node, id, scopes).map_err(|message| {
            astra_error(
                &node.manifest_rel,
                &format!("decisions.{id}.from"),
                &message,
            )
        })?;
        let endpoint = LocalGraphId::astra_decision(root_key, &scope, id);
        let name = effective.label.as_deref().unwrap_or(id);
        let mut object = astra_object("Decision", id, name, &scope);
        insert_string(&mut object, "rationale", effective.rationale.as_deref());
        insert_strings(&mut object, "tags", &effective.tags);
        insert_string(&mut object, "default", effective.default.as_deref());
        insert_strings(&mut object, "when", &decision.when);
        insert_string(&mut object, "from", decision.from.as_deref());
        if decision.from.is_some() {
            let (resolved_scope, resolved_id) =
                defining_decision(node, id, scopes).map_err(|message| {
                    astra_error(
                        &node.manifest_rel,
                        &format!("decisions.{id}.from"),
                        &message,
                    )
                })?;
            object.insert(
                "resolvedTo".to_string(),
                Primitive::String(LocalGraphId::astra_decision(
                    root_key,
                    &resolved_scope,
                    &resolved_id,
                )),
            );
        }
        builder.add_schema_node(&endpoint, Node::Object(object));
        builder.add_containment(
            &endpoint,
            &analysis_id,
            vec![declared_evidence(
                node,
                &format!("decisions.{id}"),
                Some(id),
                decision.from.as_deref(),
            )],
        );
        if decision.from.is_none() {
            for (option_id, option) in &decision.options {
                let option_endpoint = LocalGraphId::astra_option(root_key, &scope, id, option_id);
                let mut object = astra_object(
                    "Option",
                    option_id,
                    option.label.as_deref().unwrap_or(option_id),
                    &scope,
                );
                insert_string(&mut object, "description", option.description.as_deref());
                object.insert("decision".to_string(), Primitive::String(endpoint.clone()));
                insert_strings(&mut object, "insights", &option.insights);
                insert_strings(&mut object, "requires", &option.requires);
                insert_strings(&mut object, "incompatibleWith", &option.incompatible_with);
                if let Some(excluded) = option.excluded {
                    object.insert("excluded".to_string(), Primitive::Boolean(excluded));
                }
                insert_string(
                    &mut object,
                    "excludedReason",
                    option.excluded_reason.as_deref(),
                );
                builder.add_schema_node(&option_endpoint, Node::Object(object));
                builder.add_containment(
                    &option_endpoint,
                    &endpoint,
                    vec![declared_evidence(
                        node,
                        &format!("decisions.{id}.options.{option_id}"),
                        Some(option_id),
                        None,
                    )],
                );
                for insight in &option.insights {
                    let (insight_scope, insight_id) = resolve_prior_insight(node, insight, scopes)
                        .map_err(|message| {
                            astra_error(
                                &node.manifest_rel,
                                &format!("decisions.{id}.options.{option_id}.insights"),
                                &message,
                            )
                        })?;
                    builder.add_edge_with_evidence(
                        LocalGraphId::astra_insight(
                            root_key,
                            &insight_scope,
                            "prior_insights",
                            &insight_id,
                        ),
                        &option_endpoint,
                        GraphEdgeKind::Supports,
                        vec![declared_evidence(
                            node,
                            &format!("decisions.{id}.options.{option_id}.insights"),
                            Some(option_id),
                            Some(insight),
                        )],
                    );
                }
            }
        }
        decision_endpoints.insert(id.clone(), endpoint);
    }

    project_insights(builder, root_key, node, scopes)?;

    for (index, output) in node.analysis.outputs.iter().enumerate() {
        let output_id = LocalGraphId::astra_output(root_key, &scope, &output.id);
        let (effective_node, effective_output) = resolve_effective_output(node, output, scopes)
            .map_err(|message| {
                astra_error(
                    &node.manifest_rel,
                    &format!("outputs[{index}].from"),
                    &message,
                )
            })?;
        add_output_node(builder, &output_id, output, effective_output);
        builder.add_containment(
            &output_id,
            &analysis_id,
            vec![declared_evidence(
                node,
                &format!("outputs[{index}]"),
                Some(&output.id),
                output.from.as_deref(),
            )],
        );

        if let Some(from) = &output.from {
            let (child_scope, child_output) =
                resolve_output_from(node, from, scopes).map_err(|message| {
                    astra_error(
                        &node.manifest_rel,
                        &format!("outputs[{index}].from"),
                        &message,
                    )
                })?;
            let child_id = LocalGraphId::astra_output(root_key, &child_scope, &child_output);
            builder.add_derivation(
                child_id,
                &output_id,
                vec![declared_evidence(
                    node,
                    &format!("outputs[{index}].from"),
                    Some(&output.id),
                    Some(from),
                )],
            );
            if let Some(target) = effective_output.target.as_deref()
                && let Some(endpoint) = project_output_target(
                    builder,
                    workspace_root,
                    effective_node,
                    resource_id,
                    target,
                )
            {
                let evidence = vec![target_evidence(
                    node,
                    &format!("outputs[{index}].from"),
                    &output.id,
                    target,
                )];
                if endpoint.is_remote {
                    builder.add_send(&output_id, &endpoint.id, evidence);
                } else {
                    builder.add_write(&output_id, &endpoint.id, evidence);
                }
            }
            continue;
        }

        let unit_id = LocalGraphId::workflow_unit(&format!("{root_key}#{scope}"), &output.id);
        let mut unit = Function::new(
            output.label.clone().unwrap_or_else(|| output.id.clone()),
            vec![],
        );
        unit.id = Some(unit_id.clone());
        builder.add_schema_node(&unit_id, Node::Function(unit));
        let output_field = format!("outputs[{index}]");
        builder.add_declaration(
            &manifest_id,
            &unit_id,
            vec![declared_evidence(
                node,
                &output_field,
                Some(&output.id),
                None,
            )],
        );
        builder.add_containment(
            &unit_id,
            &analysis_id,
            vec![declared_evidence(
                node,
                &output_field,
                Some(&output.id),
                None,
            )],
        );
        let generation_field = if output
            .recipe
            .as_ref()
            .and_then(|recipe| recipe.command.as_ref())
            .is_some()
        {
            format!("{output_field}.recipe.command")
        } else {
            output_field.clone()
        };
        builder.add_generation(
            &unit_id,
            &output_id,
            vec![declared_evidence(
                node,
                &generation_field,
                Some(&output.id),
                None,
            )],
        );

        let target_endpoint = output.target.as_deref().and_then(|target| {
            project_output_target(builder, workspace_root, node, resource_id, target)
        });
        if let (Some(target), Some(endpoint)) = (output.target.as_deref(), target_endpoint.as_ref())
        {
            let field_path = format!("{output_field}.target");
            let evidence = vec![target_evidence(node, &field_path, &output.id, target)];
            if endpoint.is_remote {
                builder.add_send(&output_id, &endpoint.id, evidence.clone());
                builder.add_send(&unit_id, &endpoint.id, evidence);
            } else {
                builder.add_write(&output_id, &endpoint.id, evidence.clone());
                builder.add_generation(&unit_id, &endpoint.id, evidence);
            }
        }

        if let Some(command) = output
            .recipe
            .as_ref()
            .and_then(|recipe| recipe.command.as_deref())
            && let Some(script) = crate::code::direct_source_script(command)
            && let Some(rel) = local_source_rel(workspace_root, &node.manifest_rel, &script)
            && let Some(script_id) = resource_id(&rel)
        {
            let field_path = format!("{output_field}.recipe.command");
            let evidence = vec![recipe_script_evidence(
                node,
                &field_path,
                &output.id,
                &script,
            )];
            builder.add_edge_with_evidence(
                &script_id,
                &unit_id,
                GraphEdgeKind::UsedBy,
                evidence.clone(),
            );
            builder.add_generation(&script_id, &output_id, evidence);
            if let (Some(target), Some(endpoint)) =
                (output.target.as_deref(), target_endpoint.as_ref())
            {
                let evidence = vec![recipe_target_evidence(
                    node,
                    &field_path,
                    &output.id,
                    &script,
                    target,
                )];
                if endpoint.is_remote {
                    builder.add_send(&script_id, &endpoint.id, evidence);
                } else {
                    builder.add_generation(&script_id, &endpoint.id, evidence);
                }
            }
        }

        for input in &output.inputs {
            let endpoints = input_endpoints.get(input).cloned().unwrap_or_else(|| {
                let (dependency_scope, dependency_id) =
                    resolve_output_dependency(node, input, scopes)
                        .unwrap_or_else(|_| (scope.clone(), input.clone()));
                vec![InputEndpoint {
                    id: LocalGraphId::astra_output(root_key, &dependency_scope, &dependency_id),
                    is_remote: false,
                }]
            });
            for endpoint in endpoints {
                let evidence = vec![declared_evidence(
                    node,
                    &format!("{output_field}.inputs"),
                    Some(&output.id),
                    Some(input),
                )];
                if endpoint.is_remote {
                    builder.add_receive(endpoint.id, &unit_id, evidence);
                } else {
                    builder.add_read(endpoint.id, &unit_id, evidence);
                }
            }
        }
        for decision in &output.decisions {
            if let Some(decision_id) = decision_endpoints.get(decision) {
                builder.add_edge_with_evidence(
                    decision_id,
                    &unit_id,
                    GraphEdgeKind::Configures,
                    vec![declared_evidence(
                        node,
                        &format!("{output_field}.decisions"),
                        Some(&output.id),
                        Some(decision),
                    )],
                );
            }
        }
        for condition in &output.when {
            let (decision, ..) = condition_target(condition).map_err(|message| {
                astra_error(
                    &node.manifest_rel,
                    &format!("{output_field}.when"),
                    &message,
                )
            })?;
            if output.decisions.iter().any(|declared| declared == decision) {
                continue;
            }
            if let Some(decision_id) = decision_endpoints.get(decision) {
                builder.add_edge_with_evidence(
                    decision_id,
                    &unit_id,
                    GraphEdgeKind::Configures,
                    vec![declared_evidence(
                        node,
                        &format!("{output_field}.when"),
                        Some(&output.id),
                        Some(condition),
                    )],
                );
            }
        }

        if let Some(container) = output
            .recipe
            .as_ref()
            .and_then(|recipe| recipe.container.as_ref())
            .or(node.analysis.container.as_ref())
        {
            let container_field = if output
                .recipe
                .as_ref()
                .and_then(|recipe| recipe.container.as_ref())
                .is_some()
            {
                format!("{output_field}.recipe.container")
            } else {
                "container".to_string()
            };
            let container_id =
                add_container(builder, workspace_root, node, resource_id, container)?;
            builder.add_edge_with_evidence(
                container_id,
                &unit_id,
                GraphEdgeKind::Configures,
                vec![declared_evidence(
                    node,
                    &container_field,
                    Some(&output.id),
                    Some(container),
                )],
            );
        }
    }

    for child in node.children.values() {
        project_analysis(
            builder,
            workspace_root,
            root_key,
            child,
            scopes,
            resource_id,
            Some(&analysis_id),
        )?;
    }
    Ok(())
}

fn project_insights(
    builder: &mut GraphBuilder,
    root_key: &str,
    node: &AnalysisNode,
    scopes: &BTreeMap<String, &AnalysisNode>,
) -> Result<()> {
    let scope = node.scope_string();
    for (collection, insights) in [
        ("prior_insights", &node.analysis.prior_insights),
        ("findings", &node.analysis.findings),
    ] {
        for (insight_id, insight) in insights {
            let claim_id = LocalGraphId::astra_insight(root_key, &scope, collection, insight_id);
            let mut claim = Claim::new(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                Text::new(insight.claim.clone().into()),
            )]))]);
            claim.id = Some(claim_id.clone());
            claim.label = insight.label.clone();
            claim.options.name = insight.label.clone().or_else(|| Some(insight_id.clone()));
            let mut metadata = Object::from([
                ("createdAt", Primitive::String(insight.created_at.clone())),
                ("collection", Primitive::String(collection.to_string())),
            ]);
            if let Some(derived) = insight.derived {
                metadata.insert("derived".to_string(), Primitive::Boolean(derived));
            }
            insert_string(&mut metadata, "scope", insight.scope.as_deref());
            insert_strings(&mut metadata, "tags", &insight.tags);
            insert_string(&mut metadata, "notes", insight.notes.as_deref());
            claim.options.extra = Some(metadata);
            builder.add_schema_node(&claim_id, Node::Claim(claim));
            builder.add_containment(
                &claim_id,
                LocalGraphId::astra_analysis(root_key, &scope),
                vec![declared_evidence(
                    node,
                    &format!("{collection}.{insight_id}"),
                    Some(insight_id),
                    None,
                )],
            );

            for evidence in &insight.evidence {
                let evidence_id = LocalGraphId::astra_evidence(
                    root_key,
                    &scope,
                    collection,
                    insight_id,
                    &evidence.id,
                );
                let mut evidence_node = Evidence::new(vec![]);
                evidence_node.id = Some(evidence_id.clone());
                evidence_node.options.name = Some(evidence.id.clone());
                let mut metadata = Object::new();
                if let Some(doi) = &evidence.doi {
                    evidence_node.doi = bare_doi(doi).map(ToString::to_string);
                }
                insert_string(&mut metadata, "artifact", evidence.artifact.as_deref());
                insert_string(&mut metadata, "snapshot", evidence.snapshot.as_deref());
                insert_string(
                    &mut metadata,
                    "sourceCommit",
                    evidence.source_commit.as_deref(),
                );
                if let Some(version) = evidence.version {
                    metadata.insert("version".to_string(), Primitive::Integer(version as i64));
                }
                if let Some(quote) = &evidence.quote {
                    let mut selector =
                        Object::from([("exact", Primitive::String(quote.exact.clone()))]);
                    insert_string(&mut selector, "prefix", quote.prefix.as_deref());
                    insert_string(&mut selector, "suffix", quote.suffix.as_deref());
                    metadata.insert("quote".to_string(), Primitive::Object(selector));
                }
                if let Some(location) = &evidence.location {
                    let mut selector = Object::new();
                    insert_string(&mut selector, "value", location.value.as_deref());
                    if let Some(page) = location.page {
                        selector.insert("page".to_string(), Primitive::Integer(page as i64));
                    }
                    metadata.insert("location".to_string(), Primitive::Object(selector));
                }
                evidence_node.options.extra = (!metadata.is_empty()).then_some(metadata);
                builder.add_schema_node(&evidence_id, Node::Evidence(evidence_node));
                builder.add_containment(
                    &evidence_id,
                    &claim_id,
                    vec![declared_evidence(
                        node,
                        &format!("{collection}.{insight_id}.evidence"),
                        Some(insight_id),
                        Some(&evidence.id),
                    )],
                );
                builder.add_edge_with_evidence(
                    &evidence_id,
                    &claim_id,
                    GraphEdgeKind::Supports,
                    vec![declared_evidence(
                        node,
                        &format!("{collection}.{insight_id}.evidence"),
                        Some(insight_id),
                        Some(&evidence.id),
                    )],
                );

                if let Some(doi) = evidence.doi.as_deref().and_then(bare_doi) {
                    let doi_id = LocalGraphId::resource(&format!("doi:{doi}"));
                    let mut work = CreativeWork::new();
                    work.doi = Some(doi.to_string());
                    work.options.url = Some(doi_url(doi));
                    builder.add_schema_node(&doi_id, Node::CreativeWork(work));
                    builder.add_edge_with_evidence(
                        &doi_id,
                        &evidence_id,
                        GraphEdgeKind::CitedBy,
                        vec![declared_evidence(
                            node,
                            &format!("{collection}.{insight_id}.evidence"),
                            Some(insight_id),
                            Some(doi),
                        )],
                    );
                }
                if let Some(artifact) = &evidence.artifact {
                    let (artifact_scope, artifact_id) =
                        resolve_output_dependency(node, artifact, scopes).map_err(|message| {
                            astra_error(
                                &node.manifest_rel,
                                &format!("{collection}.{insight_id}.evidence"),
                                &message,
                            )
                        })?;
                    builder.add_edge_with_evidence(
                        LocalGraphId::astra_output(root_key, &artifact_scope, &artifact_id),
                        &evidence_id,
                        GraphEdgeKind::Grounds,
                        vec![declared_evidence(
                            node,
                            &format!("{collection}.{insight_id}.evidence"),
                            Some(insight_id),
                            Some(artifact),
                        )],
                    );
                }
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn project_input(
    builder: &mut GraphBuilder,
    workspace_root: &Path,
    root_key: &str,
    node: &AnalysisNode,
    scopes: &BTreeMap<String, &AnalysisNode>,
    resource_id: impl Fn(&WorkspaceRelPath) -> Option<String> + Copy,
    index: usize,
    input: &Input,
) -> Result<Vec<InputEndpoint>> {
    let scope = node.scope_string();
    if let Some(from) = &input.from {
        let (target_scope, target_id) =
            resolve_input_from(node, from, scopes).map_err(|message| {
                astra_error(
                    &node.manifest_rel,
                    &format!("inputs[{index}].from"),
                    &message,
                )
            })?;
        let target = scopes.get(&target_scope).copied().ok_or_else(|| {
            astra_error(
                &node.manifest_rel,
                &format!("inputs[{index}].from"),
                "resolved scope disappeared",
            )
        })?;
        if let Some((target_index, target_input)) = target
            .analysis
            .inputs
            .iter()
            .enumerate()
            .find(|(_, item)| item.id == target_id)
        {
            return project_input(
                builder,
                workspace_root,
                root_key,
                target,
                scopes,
                resource_id,
                target_index,
                target_input,
            );
        }
        return Ok(vec![InputEndpoint {
            id: LocalGraphId::astra_output(root_key, &target_scope, &target_id),
            is_remote: false,
        }]);
    }

    if input.r#type.as_deref() == Some("analysis") {
        let ref_value = input.r#ref.as_deref().unwrap_or(&input.id);
        let analysis_ref_id = LocalGraphId::astra_analysis_ref(root_key, &scope, &input.id);
        let mut work = CreativeWork::new();
        work.id = Some(analysis_ref_id.clone());
        work.work_type = Some(CreativeWorkType::Workflow);
        work.options.name = input.label.clone().or_else(|| Some(input.id.clone()));
        work.options.description = input.description.clone();
        if has_non_local_uri_scheme(ref_value) {
            work.options.url = Some(ref_value.to_string());
        } else {
            work.options.path = Some(ref_value.to_string());
        }
        work.options.version = input.ref_version.clone().map(StringOrNumber::String);
        builder.add_schema_node(&analysis_ref_id, Node::CreativeWork(work));
        if input.use_outputs.is_empty() {
            return Ok(vec![InputEndpoint {
                id: analysis_ref_id,
                is_remote: has_remote_uri_scheme(ref_value),
            }]);
        }
        let is_remote = has_remote_uri_scheme(ref_value);
        let endpoints = input
            .use_outputs
            .iter()
            .map(|output| {
                let id = LocalGraphId::astra_output(
                    root_key,
                    &format!("{scope}.ref.{}", input.id),
                    output,
                );
                let mut work = CreativeWork::new();
                work.id = Some(id.clone());
                work.options.name = Some(output.clone());
                builder.add_schema_node(&id, Node::CreativeWork(work));
                builder.add_containment(
                    &id,
                    &analysis_ref_id,
                    vec![declared_evidence(
                        node,
                        &format!("inputs[{index}].use_outputs"),
                        Some(&input.id),
                        Some(output),
                    )],
                );
                InputEndpoint { id, is_remote }
            })
            .collect();
        return Ok(endpoints);
    }

    if let Some(source) = &input.source {
        // A bare DOI and a relative path can have the same spelling. Prefer an
        // existing workspace file for ASTRA fields that explicitly accept paths.
        if let Some(rel) = local_source_rel(workspace_root, &node.manifest_rel, source)
            && let Some(id) = resource_id(&rel)
        {
            return Ok(vec![InputEndpoint {
                id,
                is_remote: false,
            }]);
        }
        if let Some(doi) = bare_doi(source) {
            let id = LocalGraphId::resource(&format!("doi:{doi}"));
            let mut work = CreativeWork::new();
            work.doi = Some(doi.to_string());
            work.options.url = Some(doi_url(doi));
            builder.add_schema_node(&id, Node::CreativeWork(work));
            return Ok(vec![InputEndpoint {
                id,
                is_remote: true,
            }]);
        }
        if has_non_local_uri_scheme(source) {
            let id = LocalGraphId::resource(source);
            let mut work = CreativeWork::new();
            work.options.url = Some(source.clone());
            builder.add_schema_node(&id, Node::CreativeWork(work));
            return Ok(vec![InputEndpoint {
                id,
                is_remote: has_remote_uri_scheme(source),
            }]);
        }
    }

    let id = LocalGraphId::astra_input(root_key, &scope, &input.id);
    let mut work = CreativeWork::new();
    work.id = Some(id.clone());
    work.work_type = Some(CreativeWorkType::Dataset);
    work.options.name = input.label.clone().or_else(|| Some(input.id.clone()));
    work.options.description = input.description.clone();
    if let Some(source) = &input.source {
        work.options.identifiers = Some(vec![PropertyValueOrString::String(source.clone())]);
    }
    builder.add_schema_node(&id, Node::CreativeWork(work));
    Ok(vec![InputEndpoint {
        id,
        is_remote: false,
    }])
}

/// Resolve an experimental ASTRA output target to a concrete graph endpoint.
fn project_output_target(
    builder: &mut GraphBuilder,
    workspace_root: &Path,
    node: &AnalysisNode,
    resource_id: impl Fn(&WorkspaceRelPath) -> Option<String> + Copy,
    target: &str,
) -> Option<InputEndpoint> {
    // Prefer an existing workspace path over the syntactically ambiguous bare
    // DOI form (for example, `10.1234/result.csv`).
    if let Some(rel) = local_source_rel(workspace_root, &node.manifest_rel, target)
        && let Some(id) = resource_id(&rel)
    {
        return Some(InputEndpoint {
            id,
            is_remote: false,
        });
    }

    if let Some(doi) = bare_doi(target) {
        let id = LocalGraphId::resource(&format!("doi:{doi}"));
        let mut work = CreativeWork::new();
        work.doi = Some(doi.to_string());
        work.options.url = Some(doi_url(doi));
        builder.add_schema_node(&id, Node::CreativeWork(work));
        return Some(InputEndpoint {
            id,
            is_remote: true,
        });
    }

    if has_non_local_uri_scheme(target) {
        let id = LocalGraphId::resource(target);
        let mut work = CreativeWork::new();
        work.options.url = Some(target.to_string());
        builder.add_schema_node(&id, Node::CreativeWork(work));
        return Some(InputEndpoint {
            id,
            is_remote: has_remote_uri_scheme(target),
        });
    }

    None
}

fn local_source_rel(
    workspace_root: &Path,
    manifest_rel: &WorkspaceRelPath,
    source: &str,
) -> Option<WorkspaceRelPath> {
    let parent = Path::new(manifest_rel.as_str()).parent()?;
    let normalized = normalize_relative(parent.join(source)).ok()?;
    let rel = WorkspaceRelPath::from_relative_path(&normalized).ok()?;
    workspace_root.join(rel.as_str()).exists().then_some(rel)
}

fn add_output_node(builder: &mut GraphBuilder, id: &str, output: &Output, effective: &Output) {
    if effective.r#type.as_deref() == Some("metric") {
        let mut variable =
            Variable::new(effective.label.clone().unwrap_or_else(|| output.id.clone()));
        variable.id = Some(id.to_string());
        builder.add_schema_node(id, Node::Variable(variable));
        return;
    }
    let mut work = CreativeWork::new();
    work.id = Some(id.to_string());
    work.options.name = effective.label.clone().or_else(|| Some(output.id.clone()));
    work.options.description = effective.description.clone();
    if let Some(target) = &effective.target {
        if has_non_local_uri_scheme(target) {
            work.options.url = Some(target.clone());
        } else {
            work.options.path = Some(target.clone());
        }
    }
    work.work_type = Some(match effective.r#type.as_deref() {
        Some("figure") => CreativeWorkType::Figure,
        Some("table") => CreativeWorkType::Datatable,
        Some("report") => CreativeWorkType::Report,
        _ => CreativeWorkType::Dataset,
    });
    builder.add_schema_node(id, Node::CreativeWork(work));
}

fn add_container(
    builder: &mut GraphBuilder,
    workspace_root: &Path,
    node: &AnalysisNode,
    resource_id: impl Fn(&WorkspaceRelPath) -> Option<String> + Copy,
    container: &str,
) -> Result<String> {
    if let Some(rel) = local_source_rel(workspace_root, &node.manifest_rel, container)
        && let Some(id) = resource_id(&rel)
    {
        return Ok(id);
    }
    let id = LocalGraphId::container(container);
    let mut application = SoftwareApplication::new(container.to_string());
    application.id = Some(id.clone());
    builder.add_schema_node(&id, Node::SoftwareApplication(application));
    Ok(id)
}

/// Build the single declared evidence item for an ASTRA-authored edge.
fn declared_evidence(
    node: &AnalysisNode,
    field_path: &str,
    id: Option<&str>,
    resolution: Option<&str>,
) -> GraphEvidence {
    let source = node
        .manifest_text
        .get(node.source_range.clone())
        .unwrap_or_default();
    let offset = find_yaml_field(source, field_path, id)
        .or_else(|| (field_path != "$").then_some(0))
        .map(|offset| node.source_range.start + offset);
    let mut evidence = evidence::declared_at(
        node.manifest_rel.as_str(),
        Some(&node.manifest_text),
        offset,
    );
    let mut details = Object::from([
        (
            "detector",
            Primitive::String("stencila-astra-contract".to_string()),
        ),
        (
            "manifest",
            Primitive::String(node.manifest_rel.as_str().to_string()),
        ),
        ("fieldPath", Primitive::String(field_path.to_string())),
        ("analysisScope", Primitive::String(node.scope_string())),
    ]);
    if let Some(id) = id {
        details.insert("id".to_string(), Primitive::String(id.to_string()));
    }
    if let Some(resolution) = resolution {
        details.insert(
            "resolution".to_string(),
            Primitive::String(resolution.to_string()),
        );
    }
    if let Some(output) = id.and_then(|id| node.analysis.outputs.iter().find(|out| out.id == id)) {
        if !output.when.is_empty() {
            details.insert(
                "when".to_string(),
                Primitive::Array(Array(
                    output.when.iter().cloned().map(Primitive::String).collect(),
                )),
            );
        }
        if let Some(recipe) = &output.recipe {
            if let Some(command) = &recipe.command {
                details.insert(
                    "recipeCommand".to_string(),
                    Primitive::String(command.clone()),
                );
            }
            if let Some(resources) = &recipe.resources {
                details.insert(
                    "recipeResources".to_string(),
                    Primitive::Object(resources_object(resources)),
                );
            }
            if let Some(container) = recipe
                .container
                .as_ref()
                .or(node.analysis.container.as_ref())
            {
                details.insert(
                    "effectiveContainer".to_string(),
                    Primitive::String(container.clone()),
                );
            }
        }
    } else if let Some(decision) = id.and_then(|id| node.analysis.decisions.get(id)) {
        if !decision.when.is_empty() {
            details.insert(
                "when".to_string(),
                Primitive::Array(Array(
                    decision
                        .when
                        .iter()
                        .cloned()
                        .map(Primitive::String)
                        .collect(),
                )),
            );
        }
        if !decision.tags.is_empty() {
            details.insert(
                "tags".to_string(),
                Primitive::Array(Array(
                    decision
                        .tags
                        .iter()
                        .cloned()
                        .map(Primitive::String)
                        .collect(),
                )),
            );
        }
        if let Some(rationale) = &decision.rationale {
            details.insert(
                "rationale".to_string(),
                Primitive::String(rationale.clone()),
            );
        }
    }
    evidence.options.details = Some(details);
    evidence
}

/// Build evidence for a script conservatively identified in a recipe command.
fn recipe_script_evidence(
    node: &AnalysisNode,
    field_path: &str,
    output_id: &str,
    script: &str,
) -> GraphEvidence {
    let mut evidence = declared_evidence(node, field_path, Some(output_id), None);
    if let Some(details) = evidence.options.details.as_mut() {
        details.insert("script".to_string(), Primitive::String(script.to_string()));
    }
    evidence
}

/// Build evidence for the experimental association between an output and target.
fn target_evidence(
    node: &AnalysisNode,
    field_path: &str,
    output_id: &str,
    target: &str,
) -> GraphEvidence {
    let mut evidence = declared_evidence(node, field_path, Some(output_id), None);
    add_target_evidence_details(&mut evidence, target);
    evidence
}

/// Build recipe-command evidence for generation of a concrete output target.
fn recipe_target_evidence(
    node: &AnalysisNode,
    field_path: &str,
    output_id: &str,
    script: &str,
    target: &str,
) -> GraphEvidence {
    let mut evidence = recipe_script_evidence(node, field_path, output_id, script);
    add_target_evidence_details(&mut evidence, target);
    evidence
}

fn add_target_evidence_details(evidence: &mut GraphEvidence, target: &str) {
    if let Some(details) = evidence.options.details.as_mut() {
        details.insert(
            "extension".to_string(),
            Primitive::String("stencila-output-target".to_string()),
        );
        details.insert("target".to_string(), Primitive::String(target.to_string()));
    }
}

fn find_yaml_field(source: &str, field_path: &str, id: Option<&str>) -> Option<usize> {
    if field_path == "$" {
        return None;
    }

    let key = field_path.rsplit('.').next()?;
    let item_field = key.contains('[');
    if let Some(id_offset) = id.and_then(|id| find_yaml_id(source, id)) {
        if item_field {
            return Some(id_offset);
        }
        if let Some(field_offset) = find_yaml_key_in_item(source, id_offset, key) {
            return Some(field_offset);
        }
    }

    find_yaml_key(source, key).or_else(|| id.and_then(|id| find_yaml_id(source, id)))
}

fn find_yaml_id(source: &str, id: &str) -> Option<usize> {
    find_yaml_line(source, |content| {
        content
            .strip_prefix("- ")
            .unwrap_or(content)
            .strip_prefix("id:")
            .is_some_and(|value| value.trim() == id)
    })
}

fn find_yaml_key(source: &str, key: &str) -> Option<usize> {
    find_yaml_line(source, |content| {
        content
            .split_once(':')
            .is_some_and(|(candidate, _)| candidate == key)
    })
}

fn find_yaml_key_in_item(source: &str, id_offset: usize, key: &str) -> Option<usize> {
    let item = source.get(id_offset..)?;
    let item_indent = source
        .get(..id_offset)?
        .rsplit_once('\n')
        .map_or(id_offset, |(_, prefix)| prefix.len());
    let mut offset = 0;

    for (index, line) in item.split_inclusive('\n').enumerate() {
        let content = line.trim();
        let indent = line.chars().take_while(|char| char.is_whitespace()).count();
        if index > 0
            && !content.is_empty()
            && (indent < item_indent || (indent == item_indent && content.starts_with("- ")))
        {
            break;
        }
        if content
            .split_once(':')
            .is_some_and(|(candidate, _)| candidate == key)
        {
            return Some(id_offset + offset + line.find(content).unwrap_or_default());
        }
        offset += line.len();
    }
    None
}

fn find_yaml_line(source: &str, predicate: impl Fn(&str) -> bool) -> Option<usize> {
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let content = line.trim();
        if predicate(content) {
            return Some(offset + line.find(content).unwrap_or_default());
        }
        offset += line.len();
    }
    None
}

fn astra_error(rel: &WorkspaceRelPath, field_path: &str, message: &str) -> eyre::Report {
    eyre::eyre!(
        "ASTRA manifest `{}` at `{}`: {}",
        rel.as_str(),
        field_path,
        message
    )
}
