//! Verification report types — the four-status output from the design.

use std::fmt::{self, Display};

use serde::Serialize;
use serde_json::Value;

use crate::assertion::ProvenanceAssertion;

/// Top-level structured verification report.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReport {
    pub manifest: ManifestStatus,
    pub signature: SignerStatus,
    pub asset_binding: AssetBindingStatus,
    pub provenance: ProvenanceStatus,
    pub reproducibility: ReproducibilityStatus,
    pub problems: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct ManifestStatus {
    pub present: bool,
    pub valid: bool,
    pub active: bool,
    /// Whether the manifest came from a sidecar rather than embedded bytes.
    pub from_sidecar: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerStatus {
    pub valid: bool,
    pub trusted: bool,
    /// Common name (or other identifier) extracted from the signing cert.
    pub signer: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AssetBindingStatus {
    pub valid: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceStatus {
    /// Stencila assertion present and signed.
    pub attested: bool,
    /// Schema URL declared in the assertion payload, if any.
    pub schema_url: Option<String>,
    /// Whether this build understands the declared schema URL.
    pub schema_known: bool,
    /// Parsed payload when the schema is known.
    pub assertion: Option<ProvenanceAssertion>,
    /// Raw payload as JSON, always populated when the assertion is present.
    pub raw: Option<Value>,
}

/// Reproducibility verification status. Always `NotChecked` in the MVP.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ReproducibilityStatus {
    NotChecked,
    Exact,
    Equivalent,
    Failed,
    Unavailable,
}

impl Default for ReproducibilityStatus {
    fn default() -> Self {
        Self::NotChecked
    }
}

impl Display for ReproducibilityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::NotChecked => "not checked",
            Self::Exact => "exact",
            Self::Equivalent => "equivalent",
            Self::Failed => "failed",
            Self::Unavailable => "unavailable",
        };
        f.write_str(s)
    }
}

impl Display for VerificationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Four-line status display, matching the design's UX section.
        let manifest_valid = if self.manifest.present {
            yes_no(self.manifest.valid)
        } else {
            "no manifest"
        };
        writeln!(f, "Manifest valid:           {manifest_valid}")?;

        let signer_line = match (&self.signature.signer, self.signature.trusted) {
            (Some(cn), true) => format!("yes  ({cn})"),
            (Some(cn), false) => format!("no   ({cn})"),
            (None, _) => yes_no(self.signature.trusted).to_string(),
        };
        writeln!(f, "Signer trusted:           {signer_line}")?;

        let provenance_line = if self.provenance.attested {
            match &self.provenance.schema_url {
                Some(url) if self.provenance.schema_known => {
                    format!("yes  ({url})")
                }
                Some(url) => {
                    format!("yes  ({url}, schema unknown)")
                }
                None => "yes".to_string(),
            }
        } else {
            "no".to_string()
        };
        writeln!(f, "Provenance attested:      {provenance_line}")?;

        writeln!(f, "Reproducibility verified: {}", self.reproducibility)?;

        if !self.problems.is_empty() {
            writeln!(f)?;
            for problem in &self.problems {
                writeln!(f, "- {problem}")?;
            }
        }

        Ok(())
    }
}

fn yes_no(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_report() -> VerificationReport {
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
                signer: Some("CN=Local Stencila Dev (untrusted)".to_string()),
            },
            asset_binding: AssetBindingStatus { valid: true },
            provenance: ProvenanceStatus {
                attested: true,
                schema_url: Some(crate::PROVENANCE_SCHEMA_V1.to_string()),
                schema_known: true,
                assertion: None,
                raw: None,
            },
            reproducibility: ReproducibilityStatus::NotChecked,
            problems: Vec::new(),
        }
    }

    /// Ensures a full human-readable report includes all expected signed-asset status lines.
    #[test]
    fn display_full_report() {
        let report = base_report();
        let rendered = report.to_string();
        assert!(rendered.contains("Manifest valid:           yes"));
        assert!(rendered.contains("Signer trusted:           no"));
        assert!(rendered.contains("Local Stencila Dev (untrusted)"));
        assert!(rendered.contains("Provenance attested:      yes"));
        assert!(rendered.contains("Reproducibility verified: not checked"));
    }

    /// Ensures an unsigned human-readable report makes the missing manifest explicit.
    #[test]
    fn display_no_manifest() {
        let mut report = base_report();
        report.manifest = ManifestStatus::default();
        report.signature = SignerStatus::default();
        report.provenance = ProvenanceStatus::default();
        let rendered = report.to_string();
        assert!(rendered.contains("Manifest valid:           no manifest"));
        assert!(rendered.contains("Signer trusted:           no"));
        assert!(rendered.contains("Provenance attested:      no"));
    }

    /// Ensures nested report JSON uses the same camelCase convention as the top level.
    #[test]
    fn json_uses_camel_case_for_manifest_fields() {
        let value = serde_json::to_value(base_report()).expect("serialize");
        assert!(value["manifest"].get("fromSidecar").is_some());
        assert!(value["manifest"].get("from_sidecar").is_none());
    }
}
