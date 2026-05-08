//! `stencila credentials verify` — verify a signed asset.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use eyre::Result;
use stencila_cli_utils::{AsFormat, Code, ToStdout, stencila_format::Format};

use crate::{
    report::VerificationReport,
    verifier::{CredentialVerifier, VerifyAssetRequest},
};

/// Verify the C2PA Content Credentials on an asset.
#[derive(Debug, Args)]
pub struct Cli {
    /// Path to the asset to verify.
    asset: PathBuf,

    /// Strict requirements; can be passed multiple times.
    #[arg(long = "require", value_enum)]
    require: Vec<Requirement>,

    /// Output format. Defaults to a four-status table.
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum Requirement {
    TrustedSigner,
    StencilaAssertion,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let verifier = CredentialVerifier::new();
        let request = VerifyAssetRequest {
            asset_path: self.asset,
            require_trusted_signer: self.require.contains(&Requirement::TrustedSigner),
            require_stencila_assertion: self.require.contains(&Requirement::StencilaAssertion),
        };
        let report = verifier.verify_asset(request).await?;

        match self.r#as {
            Some(format) => {
                Code::new_from(format.into(), &report)?.to_stdout();
            }
            None => {
                Code::new(Format::Text, &report.to_string()).to_stdout();
            }
        }

        if has_verification_failure(&report) {
            return Err(eyre::eyre!(
                "verification failed: {}",
                failure_summary(&report)
            ));
        }

        Ok(())
    }
}

fn has_verification_failure(report: &VerificationReport) -> bool {
    !report.manifest.present
        || !report.manifest.valid
        || !report.manifest.active
        || !report.signature.valid
        || !report.asset_binding.valid
        || (report.provenance.assertion_present
            && report.provenance.schema_known
            && report.provenance.assertion.is_none())
        || report.problems.iter().any(|p| p.starts_with("required:"))
}

fn failure_summary(report: &VerificationReport) -> String {
    let mut reasons = Vec::new();

    if report.manifest.present {
        if !report.manifest.valid {
            reasons.push("manifest invalid".to_string());
        }
        if !report.manifest.active {
            reasons.push("active manifest missing".to_string());
        }
    } else {
        reasons.push("manifest missing".to_string());
    }

    if !report.signature.valid {
        reasons.push("signature invalid".to_string());
    }

    if !report.asset_binding.valid {
        reasons.push("asset binding invalid".to_string());
    }

    if report.provenance.assertion_present
        && report.provenance.schema_known
        && report.provenance.assertion.is_none()
    {
        reasons.push("Stencila provenance assertion malformed".to_string());
    }

    reasons.extend(
        report
            .problems
            .iter()
            .filter(|problem| problem.starts_with("required:"))
            .cloned(),
    );

    if reasons.is_empty() {
        report.problems.join("; ")
    } else {
        reasons.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ProvenanceAssertion,
        report::{
            AssetBindingStatus, ManifestStatus, ProvenanceStatus, ReproducibilityStatus,
            SignerStatus, VerificationReport,
        },
    };

    use super::{failure_summary, has_verification_failure};

    fn valid_report() -> VerificationReport {
        VerificationReport {
            manifest: ManifestStatus {
                present: true,
                valid: true,
                active: true,
                from_sidecar: false,
            },
            signature: SignerStatus {
                valid: true,
                trusted: false,
                signer: Some("Local Stencila Dev (untrusted)".to_string()),
            },
            asset_binding: AssetBindingStatus { valid: true },
            provenance: ProvenanceStatus {
                assertion_present: true,
                attested: true,
                schema_url: Some(crate::PROVENANCE_SCHEMA_V1.to_string()),
                schema_known: true,
                assertion: Some(ProvenanceAssertion::new_v1("image/png", "sha256:abc")),
                raw: None,
            },
            reproducibility: ReproducibilityStatus::NotChecked,
            problems: Vec::new(),
        }
    }

    #[test]
    fn untrusted_signer_is_not_a_default_failure() {
        let report = valid_report();
        assert!(!report.signature.trusted);
        assert!(!has_verification_failure(&report));
    }

    #[test]
    fn invalid_asset_binding_is_a_default_failure() {
        let mut report = valid_report();
        report.asset_binding.valid = false;

        assert!(has_verification_failure(&report));
        assert!(failure_summary(&report).contains("asset binding invalid"));
    }

    #[test]
    fn missing_manifest_is_a_default_failure() {
        let mut report = valid_report();
        report.manifest = ManifestStatus::default();
        report.signature = SignerStatus::default();
        report.asset_binding = AssetBindingStatus::default();

        assert!(has_verification_failure(&report));
        assert!(failure_summary(&report).contains("manifest missing"));
    }

    #[test]
    fn required_problem_is_a_failure() {
        let mut report = valid_report();
        report.problems.push("required: signer trusted".to_string());

        assert!(has_verification_failure(&report));
        assert!(failure_summary(&report).contains("required: signer trusted"));
    }
}
