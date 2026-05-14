//! `stencila credentials verify` — verify a signed asset.

use std::path::PathBuf;

use clap::{Args, ValueEnum};
use eyre::{Result, bail};
use stencila_cli_utils::{
    AsFormat, Code, Tabulated, ToStdout,
    tabulated::{Attribute, Cell, Color},
};

use crate::{
    report::{ReproducibilityStatus, VerificationReport},
    verifier::{self, CredentialVerifier, VerifyAssetRequest},
};

use super::resolve_trust_anchors;

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

    /// PEM bundle of C2PA trust anchors for local signer trust checks.
    ///
    /// Can also be supplied with `STENCILA_CREDENTIALS_TRUST_ANCHORS`.
    #[arg(long, value_name = "PEM")]
    trust_anchors: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum Requirement {
    /// Require that the signing certificate chains to a trusted anchor.
    TrustedSigner,
    /// Require that the manifest carries an `org.stencila.provenance` assertion.
    StencilaAssertion,
    /// Require an exact reproducibility match.
    ///
    /// Reserved in v1: reproducibility checks are not yet implemented, so
    /// this requirement currently always reports as `unavailable`. The flag
    /// exists so that scripts and CI pipelines can adopt the contract now
    /// and pick up real comparison results once those land.
    ReproExact,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        if verifier::is_sidecar_path(&self.asset) {
            bail!(
                "Sidecar manifest '{}' cannot be verified directly. Verify the originally signed asset instead",
                self.asset.display()
            );
        }

        let trust_anchors = resolve_trust_anchors(self.trust_anchors).await?;
        let verifier = CredentialVerifier::new();
        let request = VerifyAssetRequest {
            asset_path: self.asset,
            require_trusted_signer: self.require.contains(&Requirement::TrustedSigner),
            require_stencila_assertion: self.require.contains(&Requirement::StencilaAssertion),
            require_repro_exact: self.require.contains(&Requirement::ReproExact),
            trust_anchors,
        };
        let report = verifier.verify_asset(request).await?;

        match self.r#as {
            Some(format) => {
                Code::new_from(format.into(), &report)?.to_stdout();
            }
            None => {
                print_report(&report);
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

fn print_report(report: &VerificationReport) {
    let mut table = Tabulated::new();
    table.set_header(["Check", "Status", "Details"]);

    table.add_row([
        Cell::new("Manifest valid").add_attribute(Attribute::Bold),
        if report.manifest.present {
            yes_no_cell(report.manifest.valid)
        } else {
            Cell::new("no manifest").fg(Color::Red)
        },
        manifest_details(report),
    ]);
    table.add_row([
        Cell::new("Claim signature valid").add_attribute(Attribute::Bold),
        yes_no_cell(report.signature.valid),
        Cell::new(""),
    ]);
    table.add_row([
        Cell::new("Signer trusted").add_attribute(Attribute::Bold),
        yes_no_cell(report.signature.trusted),
        signer_details(report),
    ]);
    table.add_row([
        Cell::new("Stencila provenance attested").add_attribute(Attribute::Bold),
        yes_no_cell(report.provenance.attested),
        provenance_details(report),
    ]);
    table.add_row([
        Cell::new("Stencila reproducibility checked").add_attribute(Attribute::Bold),
        reproducibility_cell(report.reproducibility),
        Cell::new(""),
    ]);

    for problem in &report.problems {
        table.add_row([
            Cell::new("Problem")
                .fg(Color::Red)
                .add_attribute(Attribute::Bold),
            Cell::new(""),
            Cell::new(problem),
        ]);
    }

    table.to_stdout();

    print_summary_table(report);
}

fn print_summary_table(report: &VerificationReport) {
    if report.summary.is_empty() {
        return;
    }

    let mut table = Tabulated::new();
    table.set_header(["Provenance", "Value"]);

    if let Some(producer) = &report.summary.producer {
        table.add_row([
            Cell::new("Producer").add_attribute(Attribute::Bold),
            Cell::new(producer),
        ]);
    }
    if let Some(asset_kind) = &report.summary.asset_kind {
        table.add_row([
            Cell::new("Asset kind").add_attribute(Attribute::Bold),
            Cell::new(asset_kind),
        ]);
    }
    if let Some(media_type) = &report.summary.media_type {
        table.add_row([
            Cell::new("Media type").add_attribute(Attribute::Bold),
            Cell::new(media_type),
        ]);
    }
    if let Some(repo) = &report.summary.source_repository {
        table.add_row([
            Cell::new("Source repository").add_attribute(Attribute::Bold),
            Cell::new(repo),
        ]);
    }
    if let Some(file) = &report.summary.source_file {
        table.add_row([
            Cell::new("Source file").add_attribute(Attribute::Bold),
            Cell::new(file),
        ]);
    }
    if let Some(range) = &report.summary.source_range {
        table.add_row([
            Cell::new("Source range").add_attribute(Attribute::Bold),
            Cell::new(range),
        ]);
    }
    if let Some(count) = report.summary.redaction_count {
        table.add_row([
            Cell::new("Privacy redactions").add_attribute(Attribute::Bold),
            Cell::new(count),
        ]);
    }

    table.to_stdout();
}

fn manifest_details(report: &VerificationReport) -> Cell {
    if report.manifest.present {
        if report.manifest.from_sidecar {
            Cell::new("sidecar")
        } else {
            Cell::new("embedded")
        }
    } else {
        Cell::new("")
    }
}

fn signer_details(report: &VerificationReport) -> Cell {
    match (&report.signature.signer, report.signature.trusted) {
        (Some(signer), true) => Cell::new(signer),
        (Some(signer), false) if report.signature.valid => {
            Cell::new(format!("{signer}; local trust not configured"))
        }
        (Some(signer), false) => Cell::new(format!("{signer}; signature invalid")),
        (None, _) => Cell::new(""),
    }
}

fn provenance_details(report: &VerificationReport) -> Cell {
    if report.provenance.attested {
        // The schema URL is long and not useful in a status table — the
        // `yes` status already conveys that the assertion was attested with
        // a known schema. Surface the URL only when this build does not
        // recognise it, where it points the reader at the right artifact.
        match &report.provenance.schema_url {
            Some(_) if report.provenance.schema_known => Cell::new(""),
            Some(url) => Cell::new(format!("schema unknown: {url}")).fg(Color::Yellow),
            None => Cell::new(""),
        }
    } else if report.provenance.assertion_present && !report.signature.valid {
        Cell::new("assertion present, claim signature invalid")
    } else if report.provenance.assertion_present && !report.asset_binding.valid {
        Cell::new("assertion present, asset binding invalid")
    } else if report.provenance.assertion_present {
        Cell::new("assertion present, not attested")
    } else {
        Cell::new("assertion not present")
    }
}

fn reproducibility_cell(status: ReproducibilityStatus) -> Cell {
    match status {
        ReproducibilityStatus::NotChecked => Cell::new(status).add_attribute(Attribute::Dim),
        ReproducibilityStatus::Exact | ReproducibilityStatus::Equivalent => {
            Cell::new(status).fg(Color::Green)
        }
        ReproducibilityStatus::Failed => Cell::new(status).fg(Color::Red),
        ReproducibilityStatus::Unavailable => Cell::new(status).fg(Color::Yellow),
    }
}

fn yes_no_cell(value: bool) -> Cell {
    if value {
        Cell::new("yes").fg(Color::Green)
    } else {
        Cell::new("no").fg(Color::Red)
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
            SignerStatus, VerificationReport, VerificationSummary,
        },
        signer::LOCAL_SIGNING_IDENTITY_COMMON_NAME,
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
                signer: Some(LOCAL_SIGNING_IDENTITY_COMMON_NAME.to_string()),
            },
            asset_binding: AssetBindingStatus { valid: true },
            provenance: ProvenanceStatus {
                assertion_present: true,
                attested: true,
                schema_url: Some(crate::PROVENANCE_SCHEMA.to_string()),
                schema_known: true,
                assertion: Some(ProvenanceAssertion::new_v1("image/png", "sha256:abc")),
                raw: None,
            },
            reproducibility: ReproducibilityStatus::NotChecked,
            summary: VerificationSummary::default(),
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
