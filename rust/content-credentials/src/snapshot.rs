//! Stencila provenance facts captured before C2PA signing.
//!
//! A snapshot is the boundary between Stencila's document, execution, workflow,
//! and export layers and this C2PA-facing crate. It intentionally uses simple
//! serializable values rather than generated Stencila Schema types so callers can
//! project only the facts they want to attest under the selected privacy profile.
//!
//! This module is an internal Stencila Rust API. It is not the C2PA wire format
//! and is not what `https://stencila.org/c2pa/v1/ProvenanceAssertion.schema.json`
//! describes. Before signing, a [`ProvenanceSnapshot`] is normalized into a
//! [`crate::ProvenanceAssertion`], and that assertion is the versioned payload
//! serialized into the `org.stencila.provenance` C2PA assertion.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Internal handoff object used to construct a signed Stencila C2PA assertion.
///
/// Other Stencila crates should populate this from document nodes, execution
/// state, workflow context, environment probes, export reports, and privacy
/// policy. The signing layer then converts it to [`crate::ProvenanceAssertion`].
///
/// This type is intentionally not the stable published assertion schema. It can
/// evolve as Stencila integration points evolve. Compatibility guarantees belong
/// to [`crate::ProvenanceAssertion`] and its schema URL.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProvenanceSnapshot {
    /// Assertion profile, such as `computational-output` or `document-export`.
    pub profile: Option<String>,

    /// Facts about the signed asset.
    pub asset: AssetSnapshot,

    /// Facts about the Stencila document node that produced or represents the asset.
    pub document: DocumentSnapshot,

    /// Software that produced the C2PA claim and Stencila assertion.
    pub producer: Option<ProducerSnapshot>,

    /// Source-control facts for the document or project.
    pub source: Option<SourceSnapshot>,

    /// Execution facts for executable outputs.
    pub execution: Option<ExecutionSnapshot>,

    /// Workflow, agent, and skill attribution supplied by an explicit provenance context.
    pub workflow: Option<WorkflowSnapshot>,

    /// Environment facts that help a verifier reproduce the output.
    pub environment: Option<EnvironmentSnapshot>,

    /// Inputs used to produce the asset.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<IoSnapshot>,

    /// Outputs produced by the same operation.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<IoSnapshot>,

    /// Compact projection of Stencila `ProvenanceCount` values.
    pub provenance_summary: Option<ProvenanceSummarySnapshot>,

    /// Reproducibility status and comparison details known at signing time.
    pub verification: Option<VerificationSnapshot>,

    /// Redactions applied while projecting private Stencila state into the assertion.
    pub privacy: Option<PrivacySnapshot>,
}

impl ProvenanceSnapshot {
    /// Create a minimal snapshot for an already-exported asset.
    #[must_use]
    pub fn for_asset(asset: AssetSnapshot) -> Self {
        Self {
            asset,
            document: DocumentSnapshot::default_file(),
            verification: Some(VerificationSnapshot::default()),
            privacy: Some(PrivacySnapshot::default()),
            ..Default::default()
        }
    }
}

/// Facts about the signed asset.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct AssetSnapshot {
    /// Stencila or C2PA-facing asset class, such as `figure`, `document`, or `dataset`.
    pub kind: String,

    /// IANA media type for the asset bytes.
    pub media_type: String,

    /// Digest of the pre-signing asset bytes.
    #[serde(alias = "sourceDigest")]
    pub digest: String,

    /// Stencila label, such as `fig:example`.
    pub label: Option<String>,

    /// Human-readable title for the asset.
    pub title: Option<String>,

    /// Asset byte length before signing.
    pub size: Option<u64>,

    /// Width for image or video assets.
    pub width: Option<u64>,

    /// Height for image or video assets.
    pub height: Option<u64>,
}

impl AssetSnapshot {
    /// Create an asset snapshot with the minimum fields Stencila signs.
    #[must_use]
    pub fn new(
        kind: impl Into<String>,
        media_type: impl Into<String>,
        digest: impl Into<String>,
    ) -> Self {
        Self {
            kind: kind.into(),
            media_type: media_type.into(),
            digest: digest.into(),
            ..Default::default()
        }
    }
}

impl Default for AssetSnapshot {
    fn default() -> Self {
        Self {
            kind: "asset".to_string(),
            media_type: String::new(),
            digest: String::new(),
            label: None,
            title: None,
            size: None,
            width: None,
            height: None,
        }
    }
}

/// Facts about the Stencila node or work represented by the signed asset.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct DocumentSnapshot {
    /// Stencila Schema node type, such as `CodeChunk`, `Figure`, or `Article`.
    pub node_type: String,

    /// Stencila node identifier.
    pub node_id: Option<String>,

    /// Path to the node within the document tree.
    pub node_path: Option<String>,

    /// Label type for labelled executable or creative-work nodes.
    pub label_type: Option<String>,

    /// Stencila label for the document node.
    pub label: Option<String>,

    /// Human-readable node title.
    pub title: Option<String>,

    /// Programming language for executable nodes.
    pub programming_language: Option<String>,

    /// Compilation digest values recorded on executable nodes.
    pub execution_digest: Option<ExecutionDigestSnapshot>,
}

impl DocumentSnapshot {
    /// Default document record for manual signing of a standalone file.
    #[must_use]
    pub fn default_file() -> Self {
        Self {
            node_type: "File".to_string(),
            ..Default::default()
        }
    }
}

impl Default for DocumentSnapshot {
    fn default() -> Self {
        Self {
            node_type: "Unknown".to_string(),
            node_id: None,
            node_path: None,
            label_type: None,
            label: None,
            title: None,
            programming_language: None,
            execution_digest: None,
        }
    }
}

/// Software that produced the assertion.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProducerSnapshot {
    pub name: Option<String>,
    pub version: Option<String>,
    pub schema_version: Option<String>,
    pub codec: Option<String>,
    pub renderer: Option<String>,
}

/// Digest values corresponding to Stencila `CompilationDigest`.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ExecutionDigestSnapshot {
    pub state_digest: Option<String>,
    pub semantic_digest: Option<String>,
    pub dependencies_digest: Option<String>,
    pub dependencies_stale: Option<u64>,
    pub dependencies_failed: Option<u64>,
}

/// Source-control facts for the signed output.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct SourceSnapshot {
    pub repository: Option<String>,
    pub commit: Option<String>,
    pub path: Option<String>,
    pub dirty: Option<bool>,
    pub patch_sha256: Option<String>,
    pub tag: Option<String>,
}

/// Execution facts for executable document nodes.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ExecutionSnapshot {
    pub status: Option<String>,
    pub ended: Option<String>,
    pub duration_ms: Option<u64>,
    pub execution_count: Option<i64>,
    pub kernel: Option<KernelSnapshot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<DependencySnapshot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<ExecutionMessageSnapshot>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct KernelSnapshot {
    pub name: Option<String>,
    pub version: Option<String>,
    pub language: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct DependencySnapshot {
    pub node_id: Option<String>,
    pub node_type: Option<String>,
    pub relation: Option<String>,
    pub digest: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ExecutionMessageSnapshot {
    pub level: Option<String>,
    pub error_type: Option<String>,
    pub message: Option<String>,
}

/// Explicit workflow, agent, and skill attribution.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct WorkflowSnapshot {
    pub run_id: Option<String>,
    pub workflow_name: Option<String>,
    pub goal_digest: Option<String>,
    pub node_id: Option<String>,
    pub thread_id: Option<String>,
    pub artifact_id: Option<String>,
    pub agent_session_id: Option<String>,
    pub agent: Option<AgentSnapshot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<DefinitionSnapshot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skill_usages: Vec<SkillUsageSnapshot>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct AgentSnapshot {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct DefinitionSnapshot {
    pub kind: Option<String>,
    pub name: Option<String>,
    pub source_path: Option<String>,
    pub version: Option<String>,
    pub content_hash: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct SkillUsageSnapshot {
    pub name: Option<String>,
    pub source_path: Option<String>,
    pub content_hash: Option<String>,
    pub loaded_by: Option<String>,
    pub tool_call_id: Option<String>,
    pub turn_index: Option<u64>,
}

/// Environment facts selected for publication under the active privacy profile.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct EnvironmentSnapshot {
    pub container_image: Option<String>,
    pub os: Option<String>,
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub runtimes: Vec<RuntimeSnapshot>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lockfiles: Vec<FileDigestSnapshot>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct FileDigestSnapshot {
    pub path: Option<String>,
    pub sha256: Option<String>,
}

/// Input or output record.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct IoSnapshot {
    pub kind: Option<String>,
    pub name: Option<String>,
    pub uri: Option<String>,
    pub media_type: Option<String>,
    pub digest: Option<String>,
    pub version: Option<String>,
    pub access: Option<String>,
    pub redaction: Option<String>,
    pub size: Option<u64>,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub row_count: Option<u64>,
    pub column_count: Option<u64>,
    pub extra: Option<Value>,
}

/// Compact projection of Stencila provenance categories.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProvenanceSummarySnapshot {
    pub human: Option<f64>,
    pub machine: Option<f64>,
    pub ai_assisted: Option<f64>,
    pub source: Option<String>,
    pub schema_version: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<ProvenanceCategorySnapshot>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProvenanceCategorySnapshot {
    pub category: String,
    pub character_count: u64,
    pub character_percent: Option<f64>,
}

/// Reproducibility status and comparison details known when signing.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct VerificationSnapshot {
    pub reproducibility_status: String,
    pub policy: Option<String>,
    pub verified_by: Option<String>,
    pub verified_at: Option<String>,
    pub comparison: Option<Value>,
}

impl Default for VerificationSnapshot {
    fn default() -> Self {
        Self {
            reproducibility_status: "not-checked".to_string(),
            policy: None,
            verified_by: None,
            verified_at: None,
            comparison: None,
        }
    }
}

/// Privacy signals and redactions applied while building the assertion.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct PrivacySnapshot {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub redactions: Vec<RedactionSnapshot>,
    pub contains_personal_data: bool,
    pub contains_secrets: bool,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct RedactionSnapshot {
    pub field: Option<String>,
    pub reason: Option<String>,
}
