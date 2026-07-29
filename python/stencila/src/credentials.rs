//! Keep credential cryptography behind a small synchronous Python boundary.
//!
//! Signing and verification remain native so expensive filesystem and
//! cryptographic work can release the interpreter lock. JSON keeps the binding
//! independent of the public Python result models.

use std::path::PathBuf;

use pyo3::prelude::*;
use serde::Serialize;
use stencila_content_credentials::{
    CredentialProducer, CredentialSignerConfig, CredentialVerifier, PreparedSignAssetRequest,
    VerifyAssetRequest, init_local_signing_identity, media,
};
use stencila_schema::Graph;

use crate::{
    graph::{PreparedGraph, parse_profile},
    utilities::{from_json, runtime, runtime_error, to_json},
};

/// Stable JSON response returned after signing a prepared asset.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SignPreparedResponse {
    path: PathBuf,
    graph: Graph,
    manifest_kind: &'static str,
    manifest_id: Option<String>,
    sidecar_path: Option<PathBuf>,
    media_type: String,
    source_digest: String,
    signed_digest: String,
    signing_mode: &'static str,
    profile: &'static str,
    warnings: Vec<String>,
}

/// Stable JSON response returned after initializing a local identity.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InitIdentityResponse {
    cert_path: PathBuf,
    key_path: PathBuf,
    created: bool,
    common_name: String,
}

/// Register native credential operations under the Python extension module.
pub fn module(stencila: &Bound<'_, PyModule>) -> PyResult<()> {
    let credentials = PyModule::new(stencila.py(), "credentials")?;
    credentials.add_function(wrap_pyfunction!(sign_prepared, &credentials)?)?;
    credentials.add_function(wrap_pyfunction!(init, &credentials)?)?;
    credentials.add_function(wrap_pyfunction!(verify, &credentials)?)?;
    credentials.add_function(wrap_pyfunction!(inspect, &credentials)?)?;
    stencila.add("credentials", credentials)
}

#[pyfunction]
#[pyo3(signature = (
    input_path, output_path, prepared, title=None, profile="public",
    cert=None, key=None, tsa_url=None
))]
#[allow(clippy::too_many_arguments)]
/// Sign bytes with the provenance payload prepared by the graph binding.
///
/// Consuming the prepared payload avoids rescanning the workspace between
/// inspection and signing, when files or selection context could have changed.
fn sign_prepared(
    py: Python<'_>,
    input_path: String,
    output_path: String,
    prepared: String,
    title: Option<String>,
    profile: &str,
    cert: Option<String>,
    key: Option<String>,
    tsa_url: Option<String>,
) -> PyResult<String> {
    let profile = parse_profile(profile)?;
    let prepared: PreparedGraph = from_json(&prepared)?;
    let input_path = PathBuf::from(input_path);
    let output_path = PathBuf::from(output_path);
    let cert = cert.map(PathBuf::from);
    let key = key.map(PathBuf::from);

    py.detach(move || {
        let source_digest = media::sha256_file(&input_path).map_err(runtime_error)?;
        if source_digest != prepared.source_digest {
            return Err(runtime_error(eyre::eyre!(
                "signing input changed after provenance preparation: expected {}, found {}",
                prepared.source_digest,
                source_digest
            )));
        }
        let signer = CredentialSignerConfig::resolve_with_options(cert, key, tsa_url)
            .map_err(runtime_error)?;
        let producer = CredentialProducer::new(signer);
        let embed = media::embed_supported(&prepared.media_type);
        let title = title
            .or_else(|| {
                output_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| "Signed asset".to_string());
        let result = runtime()
            .block_on(producer.sign_prepared_asset(PreparedSignAssetRequest {
                input_path,
                media_type: prepared.media_type.clone(),
                output_path: Some(output_path),
                title,
                assertion: prepared.graph.clone(),
                ingredients: prepared.ingredients,
                embed,
                soft_bindings: Vec::new(),
                credential_profile: profile,
            }))
            .map_err(runtime_error)?;
        let signed_digest = media::sha256_file(&result.asset_path).map_err(runtime_error)?;
        let mut warnings = prepared.warnings;
        warnings.extend(result.warnings);
        to_json(&SignPreparedResponse {
            path: result.asset_path,
            graph: prepared.graph,
            manifest_kind: if embed { "embedded" } else { "sidecar" },
            manifest_id: result.manifest_id,
            sidecar_path: result.sidecar_path,
            media_type: prepared.media_type,
            source_digest: prepared.source_digest,
            signed_digest,
            signing_mode: "local",
            profile: profile.label(),
            warnings,
        })
    })
}

#[pyfunction]
#[pyo3(signature = (force=false))]
/// Create the local identity needed for zero-configuration signing.
fn init(py: Python<'_>, force: bool) -> PyResult<String> {
    py.detach(move || {
        let result = init_local_signing_identity(force).map_err(runtime_error)?;
        to_json(&InitIdentityResponse {
            cert_path: result.cert_path,
            key_path: result.key_path,
            created: result.created,
            common_name: result.common_name,
        })
    })
}

#[pyfunction]
#[pyo3(signature = (
    path, require_trusted_signer=false, require_stencila_assertion=false
))]
/// Verify distinct credential guarantees without conflating trust and validity.
///
/// Keeping the policy switches explicit lets callers require a trusted signer
/// or a Stencila assertion independently of basic manifest validation.
fn verify(
    py: Python<'_>,
    path: String,
    require_trusted_signer: bool,
    require_stencila_assertion: bool,
) -> PyResult<String> {
    py.detach(move || {
        let report = runtime()
            .block_on(CredentialVerifier::new().verify_asset(VerifyAssetRequest {
                asset_path: PathBuf::from(path),
                require_trusted_signer,
                require_stencila_assertion,
                require_repro_exact: false,
                trust_anchors: None,
            }))
            .map_err(runtime_error)?;
        to_json(&report)
    })
}

#[pyfunction]
/// Expose the raw reader result for diagnostics and advanced integrations.
fn inspect(py: Python<'_>, path: String) -> PyResult<String> {
    py.detach(move || {
        let value = runtime()
            .block_on(CredentialVerifier::new().inspect_asset(&PathBuf::from(path), None))
            .map_err(runtime_error)?;
        to_json(&value)
    })
}
