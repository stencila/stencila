//! Verify C2PA-signed assets and produce a Stencila four-status report.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Cursor,
    path::{Path, PathBuf},
};

use c2pa::{
    Context, Manifest, Reader, ValidationState,
    settings::Settings,
    validation_status::ValidationStatus,
    validation_status::{
        ASSERTION_BMFFHASH_MALFORMED, ASSERTION_BMFFHASH_MATCH, ASSERTION_BMFFHASH_MISMATCH,
        ASSERTION_BOXESHASH_MALFORMED, ASSERTION_BOXHASH_MATCH, ASSERTION_BOXHASH_MISMATCH,
        ASSERTION_BOXHASH_UNKNOWN_BOX, ASSERTION_CLOUD_DATA_HARD_BINDING,
        ASSERTION_COLLECTIONHASH_INCORRECT_FILE_COUNT, ASSERTION_COLLECTIONHASH_INVALID_URI,
        ASSERTION_COLLECTIONHASH_MALFORMED, ASSERTION_COLLECTIONHASH_MATCH,
        ASSERTION_COLLECTIONHASH_MISMATCH, ASSERTION_DATAHASH_MALFORMED, ASSERTION_DATAHASH_MATCH,
        ASSERTION_DATAHASH_MISMATCH, ASSERTION_DATAHASH_REDACTED, CLAIM_SIGNATURE_INSIDE_VALIDITY,
        CLAIM_SIGNATURE_VALIDATED, HARD_BINDINGS_MISSING, HARD_BINDINGS_MULTIPLE,
        SIGNING_CREDENTIAL_UNTRUSTED,
    },
};
use serde::Serialize;
use serde_json::Value;

use crate::{
    error::{Error, Result},
    media,
    report::{
        AssetBindingStatus, ManifestStatus, ProvenanceStatus, ReproducibilityStatus, SignerStatus,
        VerificationReport, VerificationSummary,
    },
    schema::{PROVENANCE_LABEL, PROVENANCE_SCHEMA, ProvenanceAssertion},
};

/// Inputs for verifying an asset.
#[derive(Debug, Clone)]
pub struct VerifyAssetRequest {
    pub asset_path: PathBuf,
    pub require_trusted_signer: bool,
    pub require_stencila_assertion: bool,
    /// Require an exact reproducibility match.
    ///
    /// Reserved for v1: reproducibility checks are not yet implemented, so
    /// this requirement currently always reports as `unavailable` rather than
    /// triggering a real comparison.
    pub require_repro_exact: bool,
    /// Optional PEM bundle of C2PA trust anchors for local signer trust checks.
    pub trust_anchors: Option<String>,
}

/// Resource extracted from an inspected C2PA manifest store.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedResource {
    /// C2PA resource identifier, usually a `self#jumbf=...` URI.
    pub identifier: String,

    /// MIME type reported by the resource reference, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Relative output file path written below the requested resources directory.
    pub path: PathBuf,

    /// Number of bytes written to disk.
    pub bytes: usize,
}

/// Source from which a C2PA manifest store was inspected.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum C2paManifestSourceKind {
    /// The manifest was embedded in the inspected asset.
    Embedded,

    /// The manifest was loaded from a sidecar and validated against a paired asset.
    Sidecar,

    /// The manifest store was inspected directly without a paired asset.
    Standalone,
}

/// Inputs for inspecting C2PA provenance for graph extraction.
#[derive(Debug, Clone)]
pub struct InspectC2paRequest {
    /// Asset or standalone `.c2pa` path to inspect.
    pub path: PathBuf,

    /// Original asset to validate a sidecar against when `path` is a `.c2pa` file.
    pub paired_asset_path: Option<PathBuf>,

    /// Optional PEM bundle of C2PA trust anchors for local signer trust checks.
    pub trust_anchors: Option<String>,
}

/// Structured result from inspecting a C2PA manifest store.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct C2paInspection {
    /// Where the inspected manifest store came from.
    pub source_kind: C2paManifestSourceKind,

    /// Asset path whose bytes were bound to the manifest, if any.
    pub asset_path: Option<PathBuf>,

    /// Sidecar or standalone manifest path, if the manifest was not embedded.
    pub manifest_path: Option<PathBuf>,

    /// Raw `c2pa::Reader` JSON for lossless downstream projection.
    pub reader_json: Value,

    /// Verification report for the inspected manifest store.
    pub report: VerificationReport,
}

/// High-level credential verifier.
#[derive(Debug, Default)]
pub struct CredentialVerifier;

impl CredentialVerifier {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Verify an asset and return a structured report.
    ///
    /// # Errors
    ///
    /// Returns an error if the input asset does not exist or the blocking
    /// verification task fails.
    pub async fn verify_asset(&self, request: VerifyAssetRequest) -> Result<VerificationReport> {
        let VerifyAssetRequest {
            asset_path,
            require_trusted_signer,
            require_stencila_assertion,
            require_repro_exact,
            trust_anchors,
        } = request;

        if !asset_path.exists() {
            return Err(Error::InputNotFound(asset_path));
        }
        if is_sidecar_path(&asset_path) {
            return Err(Error::other(format!(
                "Sidecar manifest '{}' cannot be verified directly. Verify the originally signed asset instead",
                asset_path.display()
            )));
        }

        // c2pa Reader is sync; run on a blocking thread.
        let path_for_task = asset_path.clone();
        let report = tokio::task::spawn_blocking(move || {
            read_report(&path_for_task, trust_anchors.as_deref())
        })
        .await?;

        let mut report = report;

        if require_trusted_signer && !report.signature.trusted {
            report
                .problems
                .push("required: signer trusted (--require trusted-signer)".to_string());
        }
        if require_stencila_assertion && !report.provenance.assertion_present {
            report.problems.push(
                "required: org.stencila.provenance assertion (--require stencila-assertion)"
                    .to_string(),
            );
        }
        if require_repro_exact {
            // Reserved in v1: reproducibility checks are deferred. Surface as
            // an `unavailable` problem rather than an error so callers can
            // discover the requirement now and rely on it once comparison
            // rules land.
            report.problems.push(
                "required: repro-exact unavailable \
                 (reproducibility checks not implemented in v1)"
                    .to_string(),
            );
        }

        Ok(report)
    }

    /// Return the underlying c2pa Reader output as JSON for `inspect`.
    ///
    /// # Errors
    ///
    /// Returns an error if the c2pa SDK cannot open the asset, the reader JSON
    /// cannot be parsed, or the blocking inspection task fails.
    pub async fn inspect_asset(&self, path: &Path, trust_anchors: Option<String>) -> Result<Value> {
        let path_for_task = path.to_path_buf();
        let json_str = tokio::task::spawn_blocking(move || -> Result<String> {
            let reader = open_reader(&path_for_task, trust_anchors.as_deref())?;
            Ok(reader.json())
        })
        .await??;
        let value: Value = serde_json::from_str(&json_str)?;
        Ok(value)
    }

    /// Inspect C2PA provenance from an asset, paired sidecar, or standalone manifest store.
    ///
    /// # Errors
    ///
    /// Returns an error if the requested path does not exist, a paired asset is
    /// required but missing, the c2pa SDK cannot open the manifest store, or
    /// the reader JSON cannot be parsed.
    pub async fn inspect_c2pa(&self, request: InspectC2paRequest) -> Result<C2paInspection> {
        if !request.path.exists() {
            return Err(Error::InputNotFound(request.path));
        }
        if let Some(path) = request.paired_asset_path.as_ref()
            && !path.exists()
        {
            return Err(Error::InputNotFound(path.clone()));
        }

        tokio::task::spawn_blocking(move || inspect_c2pa_blocking(request)).await?
    }

    /// Extract binary resources referenced by an inspected C2PA manifest.
    ///
    /// The returned paths are relative to `output_dir`. A `resources.json`
    /// index is also written into `output_dir` so callers can map files back to
    /// C2PA resource identifiers.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset cannot be opened, the output directory or
    /// index cannot be written, or the blocking extraction task fails.
    pub async fn extract_inspection_resources(
        &self,
        path: &Path,
        manifest_json: &Value,
        output_dir: &Path,
        trust_anchors: Option<String>,
    ) -> Result<Vec<ExtractedResource>> {
        let resources = collect_resource_refs(manifest_json);
        let path_for_task = path.to_path_buf();
        let output_dir = output_dir.to_path_buf();

        tokio::task::spawn_blocking(move || {
            extract_resources_blocking(
                &path_for_task,
                &resources,
                &output_dir,
                trust_anchors.as_deref(),
            )
        })
        .await?
    }
}

fn extract_resources_blocking(
    path: &Path,
    resources: &BTreeMap<String, Option<String>>,
    output_dir: &Path,
    trust_anchors: Option<&str>,
) -> Result<Vec<ExtractedResource>> {
    fs::create_dir_all(output_dir)?;

    let reader = open_reader(path, trust_anchors)?;
    let mut extracted = Vec::new();

    for (identifier, format) in resources {
        let mut output = Cursor::new(Vec::new());
        let bytes = match reader.resource_to_stream(identifier, &mut output) {
            Ok(bytes) if bytes > 0 => bytes,
            Ok(_) => continue,
            Err(error) => {
                tracing::debug!("Could not extract C2PA resource `{identifier}`: {error}");
                continue;
            }
        };

        let relative_path = resource_file_name(extracted.len(), identifier, format.as_deref());
        fs::write(output_dir.join(&relative_path), output.into_inner())?;
        extracted.push(ExtractedResource {
            identifier: identifier.clone(),
            format: format.clone(),
            path: relative_path,
            bytes,
        });
    }

    let index = serde_json::to_vec_pretty(&extracted)?;
    fs::write(output_dir.join("resources.json"), index)?;

    Ok(extracted)
}

fn collect_resource_refs(value: &Value) -> BTreeMap<String, Option<String>> {
    let mut resources = BTreeMap::new();
    collect_resource_refs_into(value, &mut resources);
    resources
}

fn collect_resource_refs_into(value: &Value, resources: &mut BTreeMap<String, Option<String>>) {
    match value {
        Value::Object(object) => {
            if let Some(identifier) = object.get("identifier").and_then(Value::as_str) {
                let format = object
                    .get("format")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                resources.entry(identifier.to_string()).or_insert(format);
            }

            for value in object.values() {
                collect_resource_refs_into(value, resources);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_resource_refs_into(value, resources);
            }
        }
        _ => {}
    }
}

fn resource_file_name(index: usize, identifier: &str, format: Option<&str>) -> PathBuf {
    let base = identifier
        .rsplit(['/', '='])
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or("resource");
    let stem = sanitize_file_stem(base);
    let extension = resource_extension(format, identifier);

    PathBuf::from(format!("{:04}-{stem}.{extension}", index + 1))
}

fn sanitize_file_stem(value: &str) -> String {
    let stem = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches(['.', '_', '-'])
        .to_string();

    if stem.is_empty() {
        "resource".to_string()
    } else {
        stem
    }
}

fn resource_extension(format: Option<&str>, identifier: &str) -> &'static str {
    match format {
        Some("image/png" | "png") => "png",
        Some("image/jpeg" | "image/jpg" | "jpeg" | "jpg") => "jpg",
        Some("image/gif" | "gif") => "gif",
        Some("image/svg+xml" | "svg") => "svg",
        Some("image/webp" | "webp") => "webp",
        Some("application/c2pa") => "c2pa",
        _ if Path::new(identifier)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("c2pa")) =>
        {
            "c2pa"
        }
        _ => "bin",
    }
}

/// Inspect a C2PA manifest store using the synchronous c2pa reader APIs.
///
/// This performs the actual asset, sidecar, or standalone manifest read and
/// converts the resulting reader state into a graph-friendly inspection record.
/// The c2pa SDK reader is synchronous and may perform blocking file IO,
/// manifest parsing, validation, signature checks, and sidecar reads, so the
/// async [`CredentialVerifier::inspect_c2pa`] wrapper runs this helper with
/// `tokio::task::spawn_blocking` instead of doing that work on a Tokio worker
/// thread.
fn inspect_c2pa_blocking(request: InspectC2paRequest) -> Result<C2paInspection> {
    let InspectC2paRequest {
        path,
        paired_asset_path,
        trust_anchors,
    } = request;

    let (reader, source_kind, asset_path, manifest_path) = if is_sidecar_path(&path) {
        if let Some(asset_path) = paired_asset_path {
            let media_type = media::guess_media_type(&asset_path)?;
            let reader =
                read_with_sidecar(&asset_path, &path, &media_type, trust_anchors.as_deref())?;

            (
                reader,
                C2paManifestSourceKind::Sidecar,
                Some(asset_path),
                Some(path),
            )
        } else {
            let reader = read_standalone_manifest(&path, trust_anchors.as_deref())?;

            (reader, C2paManifestSourceKind::Standalone, None, Some(path))
        }
    } else {
        let media_type = media::guess_media_type(&path)?;
        let sidecar_path = media::sidecar_path(&path);
        let (reader, from_sidecar) =
            open_reader_with_source(&path, &sidecar_path, &media_type, trust_anchors.as_deref());
        let reader = reader?;

        if from_sidecar {
            (
                reader,
                C2paManifestSourceKind::Sidecar,
                Some(path),
                Some(sidecar_path),
            )
        } else {
            (reader, C2paManifestSourceKind::Embedded, Some(path), None)
        }
    };

    let reader_json = serde_json::from_str(&reader.json())?;
    let report = report_from_reader(
        &reader,
        matches!(source_kind, C2paManifestSourceKind::Sidecar),
    );

    Ok(C2paInspection {
        source_kind,
        asset_path,
        manifest_path,
        reader_json,
        report,
    })
}

fn read_report(asset_path: &Path, trust_anchors: Option<&str>) -> VerificationReport {
    let media_type = media::guess_media_type(asset_path).unwrap_or_default();
    let sidecar_path = media::sidecar_path(asset_path);

    let (reader_result, from_sidecar) =
        open_reader_with_source(asset_path, &sidecar_path, &media_type, trust_anchors);

    let reader = match reader_result {
        Ok(reader) => reader,
        Err(err) => {
            // Distinguish "no manifest at all" from "could have been embedded
            // and the sidecar is missing".
            let mut report = VerificationReport {
                manifest: ManifestStatus::default(),
                signature: SignerStatus::default(),
                asset_binding: AssetBindingStatus::default(),
                provenance: ProvenanceStatus::default(),
                reproducibility: ReproducibilityStatus::NotChecked,
                summary: VerificationSummary::default(),
                problems: Vec::new(),
            };

            let embedded_manifest_invalid =
                !from_sidecar && !is_missing_manifest(&err) && !is_unsupported_media_type(&err);

            if from_sidecar || embedded_manifest_invalid {
                report.manifest.present = true;
                report.manifest.from_sidecar = from_sidecar;
            }

            if !from_sidecar
                && media::could_have_embedded(&media_type)
                && is_missing_manifest(&err)
                && !sidecar_path.exists()
            {
                report.problems.push(format!(
                    "no embedded manifest found and no sidecar at {}; credentials may have been lost",
                    sidecar_path.display()
                ));
            } else if from_sidecar {
                report
                    .problems
                    .push(format!("sidecar C2PA manifest invalid: {err}"));
            } else if embedded_manifest_invalid {
                report
                    .problems
                    .push(format!("embedded C2PA manifest invalid: {err}"));
            } else {
                report.problems.push(format!("no C2PA manifest: {err}"));
            }
            return report;
        }
    };

    report_from_reader(&reader, from_sidecar)
}

fn report_from_reader(reader: &Reader, from_sidecar: bool) -> VerificationReport {
    let validation_state = reader.validation_state();
    let manifest_valid = matches!(
        validation_state,
        ValidationState::Valid | ValidationState::Trusted
    );
    let signer_trusted = matches!(validation_state, ValidationState::Trusted);

    let active = reader.active_manifest();

    let manifest = ManifestStatus {
        present: true,
        valid: manifest_valid,
        active: active.is_some(),
        from_sidecar,
    };

    let signature_valid = read_signature_validity(reader);
    let mut signature = SignerStatus {
        valid: signature_valid,
        trusted: signer_trusted,
        signer: None,
    };
    if let Some(m) = active {
        let cn = m.signature_info().and_then(|s| s.common_name.clone());
        let issuer = m.signature_info().and_then(|s| s.issuer.clone());
        signature.signer = cn.or(issuer);
    }

    let asset_binding = read_asset_binding(reader);

    let (provenance, provenance_problem) = active
        .map(|manifest| read_provenance(manifest, signature_valid, asset_binding.valid))
        .unwrap_or_default();

    let mut problems = collect_problems(reader);
    if let Some(problem) = provenance_problem {
        problems.push(problem);
    }

    let summary = provenance
        .assertion
        .as_ref()
        .map(VerificationSummary::from_assertion)
        .unwrap_or_default();

    VerificationReport {
        manifest,
        signature,
        asset_binding,
        provenance,
        reproducibility: ReproducibilityStatus::NotChecked,
        summary,
        problems,
    }
}

pub(crate) fn is_sidecar_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("c2pa"))
}

fn read_provenance(
    manifest: &Manifest,
    signature_valid: bool,
    asset_binding_valid: bool,
) -> (ProvenanceStatus, Option<String>) {
    // Try to find the assertion as an opaque JSON value first so we can
    // report unknown-schema cases without losing the payload.
    let raw_value: Option<Value> = manifest.find_assertion::<Value>(PROVENANCE_LABEL).ok();
    parse_provenance(raw_value, signature_valid, asset_binding_valid)
}

fn parse_provenance(
    raw_value: Option<Value>,
    signature_valid: bool,
    asset_binding_valid: bool,
) -> (ProvenanceStatus, Option<String>) {
    let assertion_present = raw_value.is_some();
    let mut status = ProvenanceStatus {
        assertion_present,
        attested: assertion_present && signature_valid && asset_binding_valid,
        schema_url: None,
        schema_known: false,
        assertion: None,
        raw: raw_value.clone(),
    };
    let mut problem = None;

    if let Some(value) = raw_value {
        if let Some(schema) = value.get("schema").and_then(Value::as_str) {
            status.schema_url = Some(schema.to_string());
            status.schema_known = schema == PROVENANCE_SCHEMA;
        }
        if value.get("version").and_then(Value::as_u64) == Some(1) {
            status.schema_known = true;
        }
        if status.schema_known {
            match serde_json::from_value::<ProvenanceAssertion>(value) {
                Ok(parsed) => status.assertion = Some(parsed),
                Err(err) => {
                    problem = Some(format!("{PROVENANCE_LABEL} v1 payload malformed: {err}"));
                }
            }
        }
    }

    (status, problem)
}

fn collect_problems(reader: &Reader) -> Vec<String> {
    let mut problems = Vec::new();
    if let Some(statuses) = reader.validation_status() {
        for status in statuses {
            let code = status.code();
            if is_problem_status(status) {
                let msg = status
                    .explanation()
                    .map_or_else(|| code.to_string(), |e| format!("{code}: {e}"));
                problems.push(msg);
            }
        }
    }
    problems
}

fn is_problem_status(status: &ValidationStatus) -> bool {
    !status.passed() && status.code() != SIGNING_CREDENTIAL_UNTRUSTED
}

fn read_signature_validity(reader: &Reader) -> bool {
    reader
        .validation_results()
        .and_then(|results| results.active_manifest())
        .is_some_and(|statuses| {
            let success = statuses.success();
            success
                .iter()
                .any(|status| status.code() == CLAIM_SIGNATURE_VALIDATED)
                && success
                    .iter()
                    .any(|status| status.code() == CLAIM_SIGNATURE_INSIDE_VALIDITY)
        })
}

fn read_asset_binding(reader: &Reader) -> AssetBindingStatus {
    let valid = reader
        .validation_results()
        .and_then(|results| results.active_manifest())
        .is_some_and(|statuses| {
            let has_binding_success = statuses
                .success()
                .iter()
                .any(|status| is_asset_binding_success(status.code()));
            let has_binding_failure = statuses
                .failure()
                .iter()
                .any(|status| is_asset_binding_failure(status.code()));

            has_binding_success && !has_binding_failure
        });

    AssetBindingStatus { valid }
}

fn is_asset_binding_success(code: &str) -> bool {
    matches!(
        code,
        ASSERTION_DATAHASH_MATCH
            | ASSERTION_BMFFHASH_MATCH
            | ASSERTION_BOXHASH_MATCH
            | ASSERTION_COLLECTIONHASH_MATCH
    )
}

fn is_asset_binding_failure(code: &str) -> bool {
    matches!(
        code,
        HARD_BINDINGS_MISSING
            | HARD_BINDINGS_MULTIPLE
            | ASSERTION_DATAHASH_MISMATCH
            | ASSERTION_DATAHASH_MALFORMED
            | ASSERTION_DATAHASH_REDACTED
            | ASSERTION_BMFFHASH_MISMATCH
            | ASSERTION_BMFFHASH_MALFORMED
            | ASSERTION_BOXHASH_MISMATCH
            | ASSERTION_BOXHASH_UNKNOWN_BOX
            | ASSERTION_BOXESHASH_MALFORMED
            | ASSERTION_COLLECTIONHASH_MISMATCH
            | ASSERTION_COLLECTIONHASH_INCORRECT_FILE_COUNT
            | ASSERTION_COLLECTIONHASH_INVALID_URI
            | ASSERTION_COLLECTIONHASH_MALFORMED
            | ASSERTION_CLOUD_DATA_HARD_BINDING
    )
}

fn open_reader(path: &Path, trust_anchors: Option<&str>) -> Result<Reader> {
    let media_type = media::guess_media_type(path).unwrap_or_default();
    let sidecar_path = media::sidecar_path(path);
    let (reader_result, _) =
        open_reader_with_source(path, &sidecar_path, &media_type, trust_anchors);
    reader_result
}

fn open_reader_with_source(
    asset_path: &Path,
    sidecar_path: &Path,
    media_type: &str,
    trust_anchors: Option<&str>,
) -> (Result<Reader>, bool) {
    if media::could_have_embedded(media_type) {
        match read_embedded(asset_path, media_type, trust_anchors) {
            Ok(reader) => return (Ok(reader), false),
            Err(embedded_err) => {
                if is_missing_manifest(&embedded_err) && sidecar_path.exists() {
                    return match read_with_sidecar(
                        asset_path,
                        sidecar_path,
                        media_type,
                        trust_anchors,
                    ) {
                        Ok(reader) => (Ok(reader), true),
                        Err(err) => (Err(err), true),
                    };
                }

                return (Err(embedded_err), false);
            }
        }
    }

    if sidecar_path.exists() {
        return match read_with_sidecar(asset_path, sidecar_path, media_type, trust_anchors) {
            Ok(reader) => (Ok(reader), true),
            Err(err) => (Err(err), true),
        };
    }

    (read_embedded(asset_path, media_type, trust_anchors), false)
}

fn read_embedded(
    asset_path: &Path,
    media_type: &str,
    trust_anchors: Option<&str>,
) -> Result<Reader> {
    let mut asset = File::open(asset_path)?;
    reader_with_context(trust_anchors)?
        .with_stream(media_type, &mut asset)
        .map_err(Error::C2pa)
}

fn read_with_sidecar(
    asset_path: &Path,
    sidecar_path: &Path,
    media_type: &str,
    trust_anchors: Option<&str>,
) -> Result<Reader> {
    let manifest_bytes = std::fs::read(sidecar_path)?;
    let mut asset = File::open(asset_path)?;
    reader_with_context(trust_anchors)?
        .with_manifest_data_and_stream(&manifest_bytes, media_type, &mut asset)
        .map_err(Error::C2pa)
}

fn read_standalone_manifest(path: &Path, trust_anchors: Option<&str>) -> Result<Reader> {
    let mut manifest = File::open(path)?;
    reader_with_context(trust_anchors)?
        .with_stream("application/c2pa", &mut manifest)
        .map_err(Error::C2pa)
}

fn reader_with_context(trust_anchors: Option<&str>) -> Result<Reader> {
    let mut settings = Settings::new().with_value("builder.thumbnail.enabled", false)?;

    if let Some(trust_anchors) = trust_anchors {
        settings = settings.with_value("trust.trust_anchors", trust_anchors)?;
    }

    let context = Context::new().with_settings(settings)?;
    Ok(Reader::from_context(context))
}

fn is_missing_manifest(err: &Error) -> bool {
    matches!(err, Error::C2pa(c2pa::Error::JumbfNotFound))
}

fn is_unsupported_media_type(err: &Error) -> bool {
    matches!(err, Error::C2pa(c2pa::Error::UnsupportedType))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Ensures missing Stencila provenance assertions produce an unattested status cleanly.
    #[test]
    fn parse_provenance_none() {
        let (status, problem) = parse_provenance(None, true, true);
        assert!(!status.assertion_present);
        assert!(!status.attested);
        assert!(status.schema_url.is_none());
        assert!(!status.schema_known);
        assert!(status.assertion.is_none());
        assert!(status.raw.is_none());
        assert!(problem.is_none());
    }

    /// Ensures future-schema assertions remain attested and raw-preserved without v1 parsing.
    #[test]
    fn parse_provenance_unknown_schema_url() {
        let raw = json!({
            "schema": "https://stencila.org/stencila-provenance-assertion-v999.schema.json",
            "producer": { "name": "Stencila", "version": "9.9.9" },
            "asset": { "mediaType": "image/png", "sourceDigest": "sha256:abc" }
        });
        let (status, problem) = parse_provenance(Some(raw.clone()), true, true);
        assert!(status.assertion_present);
        assert!(status.attested);
        assert_eq!(
            status.schema_url.as_deref(),
            Some("https://stencila.org/stencila-provenance-assertion-v999.schema.json")
        );
        assert!(!status.schema_known);
        assert!(status.assertion.is_none(), "no v1 parse for unknown schema");
        assert_eq!(status.raw, Some(raw));
        assert!(problem.is_none(), "unknown schema is not itself a problem");
    }

    /// Ensures malformed known-schema assertions are reported instead of silently ignored.
    #[test]
    fn parse_provenance_v1_malformed() {
        let raw = json!({
            "schema": PROVENANCE_SCHEMA,
            "asset": { "mediaType": 42, "digest": "sha256:abc" },
        });
        let (status, problem) = parse_provenance(Some(raw), true, true);
        assert!(status.assertion_present);
        assert!(status.attested);
        assert!(status.schema_known);
        assert!(status.assertion.is_none());
        let problem = problem.expect("malformed v1 must surface a problem");
        assert!(
            problem.contains(PROVENANCE_LABEL) && problem.contains("malformed"),
            "unexpected problem text: {problem}"
        );
    }

    /// Ensures an assertion is not reported as attested unless the claim signature is valid.
    #[test]
    fn parse_provenance_requires_valid_signature_for_attestation() {
        let raw = serde_json::to_value(ProvenanceAssertion::new_v1("image/png", "sha256:abc"))
            .expect("serialize");
        let (status, problem) = parse_provenance(Some(raw), false, true);

        assert!(status.assertion_present);
        assert!(!status.attested);
        assert!(status.schema_known);
        assert!(status.assertion.is_some());
        assert!(problem.is_none());
    }

    /// Ensures an assertion is not reported as attested unless the asset binding is valid.
    #[test]
    fn parse_provenance_requires_valid_asset_binding_for_attestation() {
        let raw = serde_json::to_value(ProvenanceAssertion::new_v1("image/png", "sha256:abc"))
            .expect("serialize");
        let (status, problem) = parse_provenance(Some(raw), true, false);

        assert!(status.assertion_present);
        assert!(!status.attested);
        assert!(status.schema_known);
        assert!(status.assertion.is_some());
        assert!(problem.is_none());
    }

    /// Ensures verifier settings can be loaded into a reader context.
    #[test]
    fn reader_context_accepts_settings() {
        let reader = reader_with_context(None);

        assert!(reader.is_ok());

        let reader = reader_with_context(Some(
            "-----BEGIN CERTIFICATE-----\n-----END CERTIFICATE-----",
        ));

        assert!(reader.is_ok());
    }
}
