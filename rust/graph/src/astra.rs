//! Native projection of ASTRA contracts into workspace graphs.
//!
//! ASTRA manifests are declarations. This module parses and validates their
//! contract structure, but never invokes recipes or interprets recipe commands.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    ops::Range,
    path::{Component, Path, PathBuf},
};

use eyre::{Result, WrapErr, bail};
use serde::Deserialize;
use stencila_schema::{
    Array, CreativeWork, CreativeWorkType, EnumValidator, Function, GraphEdgeKind, GraphEvidence,
    Node, Object, Parameter, Primitive, PropertyValueOrString, SoftwareApplication, StringOrNumber,
    Validator, Variable,
};

use crate::{
    GraphBuilder, evidence,
    ids::{LocalGraphId, WorkspaceRelPath},
    reference::{has_non_local_uri_scheme, has_remote_uri_scheme},
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

/// The subset of ASTRA v1 needed for contract graph projection.
///
/// Unknown fields are intentionally ignored for forward compatibility.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct Analysis {
    id: Option<String>,
    version: Option<serde_yaml::Value>,
    name: Option<String>,
    description: Option<String>,
    container: Option<String>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    decisions: BTreeMap<String, Decision>,
    analyses: BTreeMap<String, Analysis>,
    path: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
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
#[serde(default)]
struct Output {
    id: String,
    label: Option<String>,
    r#type: Option<String>,
    description: Option<String>,
    from: Option<String>,
    when: Vec<String>,
    inputs: Vec<String>,
    decisions: Vec<String>,
    recipe: Option<Recipe>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct Recipe {
    command: Option<String>,
    resources: Option<serde_yaml::Value>,
    container: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
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
#[serde(default)]
struct OptionSpec {
    #[serde(rename = "label")]
    _label: Option<String>,
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
            project_tree(builder, root, &node, resource_id)
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
        || analysis.container.is_some()
        || !analysis.inputs.is_empty()
        || !analysis.outputs.is_empty()
        || !analysis.decisions.is_empty()
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
                || !output.inputs.is_empty()
                || !output.decisions.is_empty()
                || output.recipe.is_some()
            {
                return Err(astra_error(
                    &node.manifest_rel,
                    &format!("outputs[{index}].from"),
                    "an output re-export may declare only `id`, `from`, and `when`",
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
            if let Some(recipe) = &output.recipe
                && recipe.command.as_deref().is_none_or(str::is_empty)
            {
                return Err(astra_error(
                    &node.manifest_rel,
                    &format!("outputs[{index}].recipe.command"),
                    "recipe command is required",
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
                    && !node.analysis.outputs.iter().any(|item| item.id == *input)
                {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("outputs[{index}].inputs"),
                        &format!("unresolved input or sibling output `{input}` in `{scope}`"),
                    ));
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
                if option._label.is_none() {
                    return Err(astra_error(
                        &node.manifest_rel,
                        &format!("decisions.{id}.options.{option_id}.label"),
                        "an option requires `label`",
                    ));
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
    }
    validate_output_cycles(node)
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
    let decision = node
        .analysis
        .decisions
        .get(id)
        .ok_or_else(|| format!("condition references undeclared decision `{id}`"))?;
    let Some(from) = &decision.from else {
        return Ok(decision);
    };
    let (target_scope, target_id) = resolve_decision_from(node, from, scopes)?;
    scopes
        .get(&target_scope)
        .and_then(|target| target.analysis.decisions.get(&target_id))
        .ok_or_else(|| format!("resolved decision `{from}` disappeared"))
}

fn validate_id(rel: &WorkspaceRelPath, field: &str, id: &str) -> Result<()> {
    let valid = !id.is_empty()
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

/// Project a validated ASTRA root using its scoped symbol tables.
fn project_tree(
    builder: &mut GraphBuilder,
    workspace_root: &Path,
    root: &AnalysisNode,
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
    )
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
    let mut analysis_node = CreativeWork::new();
    analysis_node.id = Some(analysis_id.clone());
    analysis_node.work_type = Some(CreativeWorkType::Workflow);
    analysis_node.options.name = node
        .analysis
        .name
        .clone()
        .or_else(|| node.analysis.id.clone());
    analysis_node.options.description = node.analysis.description.clone();
    builder.add_schema_node(&analysis_id, Node::CreativeWork(analysis_node));

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
        let (effective, resolution) = if let Some(from) = &decision.from {
            let (target_scope, target_id) =
                resolve_decision_from(node, from, scopes).map_err(|message| {
                    astra_error(
                        &node.manifest_rel,
                        &format!("decisions.{id}.from"),
                        &message,
                    )
                })?;
            let target = scopes.get(&target_scope).copied().ok_or_else(|| {
                astra_error(
                    &node.manifest_rel,
                    &format!("decisions.{id}.from"),
                    "resolved decision scope disappeared",
                )
            })?;
            let effective = target.analysis.decisions.get(&target_id).ok_or_else(|| {
                astra_error(
                    &node.manifest_rel,
                    &format!("decisions.{id}.from"),
                    "resolved decision disappeared",
                )
            })?;
            (effective, Some(from.as_str()))
        } else {
            (decision, None)
        };
        let endpoint = LocalGraphId::astra_decision(root_key, &scope, id);
        let mut parameter = Parameter::new(id.clone());
        parameter.id = Some(endpoint.clone());
        parameter.options.label = effective.label.clone();
        parameter.options.default = effective
            .default
            .as_ref()
            .map(|value| Box::new(Node::String(value.clone())));
        parameter.options.validator = Some(Validator::EnumValidator(EnumValidator::new(
            effective
                .options
                .keys()
                .map(|value| Node::String(value.clone()))
                .collect(),
        )));
        builder.add_schema_node(&endpoint, Node::Parameter(parameter));
        builder.add_containment(
            &endpoint,
            &analysis_id,
            vec![declared_evidence(
                node,
                &format!("decisions.{id}"),
                Some(id),
                resolution,
            )],
        );
        decision_endpoints.insert(id.clone(), endpoint);
    }

    for (index, output) in node.analysis.outputs.iter().enumerate() {
        let output_id = LocalGraphId::astra_output(root_key, &scope, &output.id);
        add_output_node(builder, &output_id, output);
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
        builder.add_generation(
            &unit_id,
            &output_id,
            vec![declared_evidence(
                node,
                &format!("{output_field}.recipe"),
                Some(&output.id),
                None,
            )],
        );

        for input in &output.inputs {
            let endpoints = input_endpoints.get(input).cloned().unwrap_or_else(|| {
                vec![InputEndpoint {
                    id: LocalGraphId::astra_output(root_key, &scope, input),
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
            let container_id =
                add_container(builder, workspace_root, node, resource_id, container)?;
            builder.add_edge_with_evidence(
                container_id,
                &unit_id,
                GraphEdgeKind::Configures,
                vec![declared_evidence(
                    node,
                    &format!("{output_field}.recipe.container"),
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
        if let Some(rel) = local_source_rel(workspace_root, &node.manifest_rel, source)
            && let Some(id) = resource_id(&rel)
        {
            return Ok(vec![InputEndpoint {
                id,
                is_remote: false,
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

fn add_output_node(builder: &mut GraphBuilder, id: &str, output: &Output) {
    if output.r#type.as_deref() == Some("metric") {
        let mut variable = Variable::new(output.label.clone().unwrap_or_else(|| output.id.clone()));
        variable.id = Some(id.to_string());
        builder.add_schema_node(id, Node::Variable(variable));
        return;
    }
    let mut work = CreativeWork::new();
    work.id = Some(id.to_string());
    work.options.name = output.label.clone().or_else(|| Some(output.id.clone()));
    work.options.description = output.description.clone();
    work.work_type = Some(match output.r#type.as_deref() {
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
    let offset = id
        .and_then(|id| find_yaml_id(source, id))
        .or_else(|| {
            let key = field_path.rsplit('.').next().unwrap_or(field_path);
            (key != "$").then(|| find_yaml_key(source, key)).flatten()
        })
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
            if let Some(resources) = &recipe.resources
                && let Ok(resources) = serde_yaml::to_string(resources)
            {
                details.insert(
                    "recipeResources".to_string(),
                    Primitive::String(resources.trim().to_string()),
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
            .strip_suffix(':')
            .is_some_and(|candidate| candidate == key)
    })
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
