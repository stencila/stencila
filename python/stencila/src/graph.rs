//! Keep Python graph inspection aligned with credential signing.
//!
//! The native boundary owns workspace scanning and hashing so blocking work can
//! release the interpreter lock, while Python retains its ergonomic public API.

use std::path::PathBuf;

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use stencila_content_credentials::{
    CredentialProfile, IngredientSnapshot,
    media::{guess_media_type, sha256_file},
};
use stencila_graph::{
    AssetGraphOptions, WorkspaceOptions, credential_graph_for_asset, graph_from_path,
};
use stencila_schema::Graph;

use crate::utilities::{runtime, runtime_error, to_json, value_error};

/// Carry native preparation results across the Python signing boundary.
///
/// Serializing one prepared value ensures the graph, ingredients, digest, and
/// warnings used for inspection are the same values later used for signing.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreparedGraph {
    /// The asset-centred provenance assertion.
    pub graph: Graph,
    /// Snapshots of files used to produce the asset.
    pub ingredients: Vec<IngredientSnapshot>,
    /// Non-fatal provenance gaps to expose to the caller.
    pub warnings: Vec<String>,
    /// The detected media type of the unsigned bytes.
    pub media_type: String,
    /// The digest of the unsigned bytes.
    pub source_digest: String,
}

/// How much provenance evidence a caller requires for an asset.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Provenance {
    /// Record whatever provenance the workspace supports.
    Auto,
    /// Fail unless the asset can be linked to a generating source.
    Required,
    /// Skip workspace discovery entirely.
    None,
}

/// Register native graph preparation under the Python extension module.
pub fn module(stencila: &Bound<'_, PyModule>) -> PyResult<()> {
    let graph = PyModule::new(stencila.py(), "graph")?;
    graph.add_function(wrap_pyfunction!(prepare, &graph)?)?;
    stencila.add("graph", graph)
}

#[pyfunction]
#[pyo3(signature = (
    input_path, asset_path, lookup_path, workspace, source=None, source_line=None,
    profile="public", provenance="auto", title=None
))]
#[allow(clippy::too_many_arguments)]
/// Prepare the exact provenance payload that credential signing will consume.
///
/// Workspace discovery and file hashing run without holding the interpreter
/// lock because they may perform substantial filesystem work.
fn prepare(
    py: Python<'_>,
    input_path: String,
    asset_path: String,
    lookup_path: String,
    workspace: String,
    source: Option<String>,
    source_line: Option<u64>,
    profile: &str,
    provenance: &str,
    title: Option<String>,
) -> PyResult<String> {
    let profile = parse_profile(profile)?;
    let provenance = parse_provenance(provenance)?;
    let input_path = PathBuf::from(input_path);
    let asset_path = PathBuf::from(asset_path);
    let lookup_path = PathBuf::from(lookup_path);
    let workspace = PathBuf::from(workspace);
    let source = source.map(PathBuf::from);

    py.detach(move || {
        let media_type = guess_media_type(&input_path).map_err(runtime_error)?;
        let source_digest = sha256_file(&input_path).map_err(runtime_error)?;
        let workspace_graph = if provenance == Provenance::None {
            Graph::new("asset:minimal".to_string(), Vec::new(), Vec::new())
        } else {
            runtime()
                .block_on(graph_from_path(
                    &workspace,
                    Some(WorkspaceOptions {
                        include_c2pa: false,
                        ..Default::default()
                    }),
                ))
                .map_err(runtime_error)?
        };
        let prepared = credential_graph_for_asset(
            &workspace_graph,
            &workspace,
            &asset_path,
            &source_digest,
            &media_type,
            &AssetGraphOptions {
                lookup_path: Some(lookup_path),
                source_path: source,
                source_line,
                profile,
                require_source: provenance == Provenance::Required,
                title,
            },
        )
        .map_err(runtime_error)?;
        to_json(&PreparedGraph {
            graph: prepared.graph,
            ingredients: prepared.ingredients,
            warnings: prepared.warnings,
            media_type,
            source_digest,
        })
    })
}

/// Reject unknown disclosure profiles before native processing begins.
pub(crate) fn parse_profile(profile: &str) -> PyResult<CredentialProfile> {
    match profile {
        "public" => Ok(CredentialProfile::Public),
        "private" => Ok(CredentialProfile::Private),
        "full" => Ok(CredentialProfile::Full),
        _ => Err(value_error(eyre::eyre!(
            "profile must be `public`, `private`, or `full`"
        ))),
    }
}

/// Reject unknown provenance requirements before native processing begins.
fn parse_provenance(provenance: &str) -> PyResult<Provenance> {
    match provenance {
        "auto" => Ok(Provenance::Auto),
        "required" => Ok(Provenance::Required),
        "none" => Ok(Provenance::None),
        _ => Err(value_error(eyre::eyre!(
            "provenance must be `auto`, `required`, or `none`"
        ))),
    }
}
