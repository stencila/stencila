//! Verify C2PA-signed assets and produce a Stencila four-status report.

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use c2pa::{
    Manifest, Reader, ValidationState,
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
use serde_json::Value;

use crate::{
    assertion::ProvenanceAssertion,
    error::{Error, Result},
    media,
    report::{
        AssetBindingStatus, ManifestStatus, ProvenanceStatus, ReproducibilityStatus, SignerStatus,
        VerificationReport,
    },
    schema::{PROVENANCE_LABEL, PROVENANCE_SCHEMA_V1},
};

/// Inputs for verifying an asset.
#[derive(Debug, Clone)]
pub struct VerifyAssetRequest {
    pub asset_path: PathBuf,
    pub require_trusted_signer: bool,
    pub require_stencila_assertion: bool,
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
        } = request;

        if !asset_path.exists() {
            return Err(Error::InputNotFound(asset_path));
        }

        // c2pa Reader is sync; run on a blocking thread.
        let path_for_task = asset_path.clone();
        let report = tokio::task::spawn_blocking(move || read_report(&path_for_task)).await?;

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

        Ok(report)
    }

    /// Return the underlying c2pa Reader output as JSON for `inspect`.
    ///
    /// # Errors
    ///
    /// Returns an error if the c2pa SDK cannot open the asset, the reader JSON
    /// cannot be parsed, or the blocking inspection task fails.
    pub async fn inspect_asset(&self, path: &Path) -> Result<Value> {
        let path_for_task = path.to_path_buf();
        let json_str = tokio::task::spawn_blocking(move || -> Result<String> {
            let reader = open_reader(&path_for_task)?;
            Ok(reader.json())
        })
        .await??;
        let value: Value = serde_json::from_str(&json_str)?;
        Ok(value)
    }
}

fn read_report(asset_path: &Path) -> VerificationReport {
    let media_type = media::guess_media_type(asset_path).unwrap_or_default();
    let sidecar_path = media::sidecar_path(asset_path);

    let (reader_result, from_sidecar) =
        open_reader_with_source(asset_path, &sidecar_path, &media_type);

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

    let signature_valid = read_signature_validity(&reader);
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

    let asset_binding = read_asset_binding(&reader);

    let (provenance, provenance_problem) = active
        .map(|manifest| read_provenance(manifest, signature_valid))
        .unwrap_or_default();

    let mut problems = collect_problems(&reader);
    if let Some(problem) = provenance_problem {
        problems.push(problem);
    }

    VerificationReport {
        manifest,
        signature,
        asset_binding,
        provenance,
        reproducibility: ReproducibilityStatus::NotChecked,
        problems,
    }
}

fn read_provenance(
    manifest: &Manifest,
    signature_valid: bool,
) -> (ProvenanceStatus, Option<String>) {
    // Try to find the assertion as an opaque JSON value first so we can
    // report unknown-schema cases without losing the payload.
    let raw_value: Option<Value> = manifest.find_assertion::<Value>(PROVENANCE_LABEL).ok();
    parse_provenance(raw_value, signature_valid)
}

fn parse_provenance(
    raw_value: Option<Value>,
    signature_valid: bool,
) -> (ProvenanceStatus, Option<String>) {
    let assertion_present = raw_value.is_some();
    let mut status = ProvenanceStatus {
        assertion_present,
        attested: assertion_present && signature_valid,
        schema_url: None,
        schema_known: false,
        assertion: None,
        raw: raw_value.clone(),
    };
    let mut problem = None;

    if let Some(value) = raw_value {
        if let Some(schema) = value.get("schema").and_then(Value::as_str) {
            status.schema_url = Some(schema.to_string());
            status.schema_known = schema == PROVENANCE_SCHEMA_V1;
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

fn open_reader(path: &Path) -> Result<Reader> {
    let media_type = media::guess_media_type(path).unwrap_or_default();
    let sidecar_path = media::sidecar_path(path);
    let (reader_result, _) = open_reader_with_source(path, &sidecar_path, &media_type);
    reader_result
}

fn open_reader_with_source(
    asset_path: &Path,
    sidecar_path: &Path,
    media_type: &str,
) -> (Result<Reader>, bool) {
    if media::could_have_embedded(media_type) {
        match read_embedded(asset_path, media_type) {
            Ok(reader) => return (Ok(reader), false),
            Err(embedded_err) => {
                if is_missing_manifest(&embedded_err) && sidecar_path.exists() {
                    return match read_with_sidecar(asset_path, sidecar_path, media_type) {
                        Ok(reader) => (Ok(reader), true),
                        Err(err) => (Err(err), true),
                    };
                }

                return (Err(embedded_err), false);
            }
        }
    }

    if sidecar_path.exists() {
        return match read_with_sidecar(asset_path, sidecar_path, media_type) {
            Ok(reader) => (Ok(reader), true),
            Err(err) => (Err(err), true),
        };
    }

    (read_embedded(asset_path, media_type), false)
}

fn read_embedded(asset_path: &Path, media_type: &str) -> Result<Reader> {
    let mut asset = File::open(asset_path)?;
    #[allow(deprecated)]
    Reader::from_stream(media_type, &mut asset).map_err(Error::C2pa)
}

fn read_with_sidecar(asset_path: &Path, sidecar_path: &Path, media_type: &str) -> Result<Reader> {
    let manifest_bytes = std::fs::read(sidecar_path)?;
    let mut asset = File::open(asset_path)?;
    #[allow(deprecated)]
    Reader::from_manifest_data_and_stream(&manifest_bytes, media_type, &mut asset)
        .map_err(Error::C2pa)
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
        let (status, problem) = parse_provenance(None, true);
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
            "schema": "https://stencila.org/v999/ProvenanceAssertion.schema.json",
            "producer": { "name": "Stencila", "version": "9.9.9" },
            "asset": { "mediaType": "image/png", "sourceDigest": "sha256:abc" }
        });
        let (status, problem) = parse_provenance(Some(raw.clone()), true);
        assert!(status.assertion_present);
        assert!(status.attested);
        assert_eq!(
            status.schema_url.as_deref(),
            Some("https://stencila.org/v999/ProvenanceAssertion.schema.json")
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
            "schema": PROVENANCE_SCHEMA_V1,
            "producer": { "name": "Stencila" }, // missing required `version`
        });
        let (status, problem) = parse_provenance(Some(raw), true);
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
        let raw = json!({
            "schema": PROVENANCE_SCHEMA_V1,
            "producer": { "name": "Stencila", "version": "9.9.9" },
            "asset": { "mediaType": "image/png", "sourceDigest": "sha256:abc" }
        });
        let (status, problem) = parse_provenance(Some(raw), false);

        assert!(status.assertion_present);
        assert!(!status.attested);
        assert!(status.schema_known);
        assert!(status.assertion.is_some());
        assert!(problem.is_none());
    }
}
