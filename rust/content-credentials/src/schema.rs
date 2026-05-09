//! Wire schema for the Stencila C2PA provenance assertion.
//!
//! This module is the source of truth for the JSON payload serialized into the
//! `org.stencila.provenance` C2PA assertion. These types should evolve more
//! conservatively than the internal snapshot types in [`crate::snapshot`].

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

/// C2PA assertion label for Stencila provenance.
///
/// Stable across normal evolution. Bumped only on a true wire-format break.
pub const PROVENANCE_LABEL: &str = "org.stencila.provenance";

/// Payload schema URL for v1 of the Stencila provenance assertion.
///
/// This follows the same shape as Stencila document JSON Schema URLs:
/// `https://stencila.org/v.../{Type}.schema.json`. Here `v1` is the
/// provenance assertion payload version, not the Stencila release version.
///
/// New optional fields and refined semantics mint a new schema URL without
/// changing [`PROVENANCE_LABEL`].
pub const PROVENANCE_SCHEMA_V1: &str =
    "https://stencila.org/c2pa/v1/ProvenanceAssertion.schema.json";

/// The serialized payload for the `org.stencila.provenance` C2PA assertion.
///
/// This is the compatibility-sensitive wire format described by
/// [`PROVENANCE_SCHEMA_V1`]. Callers usually build a
/// [`crate::ProvenanceSnapshot`] and let [`crate::ProvenanceAssertion::from_snapshot`]
/// map it into this payload before signing.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceAssertion {
    /// Schema URL identifying the payload version.
    pub schema: String,

    /// Assertion profile, such as `computational-output` or `document-export`.
    pub profile: String,

    /// Software that produced the asset.
    pub producer: ProducerRecord,

    /// The asset to which this assertion is bound.
    pub asset: AssetRecord,

    /// The Stencila document node or work represented by the signed asset.
    pub document: DocumentRecord,

    /// Source-control facts for the document or project.
    pub source: Option<SourceRecord>,

    /// Execution facts for executable outputs.
    pub execution: Option<ExecutionRecord>,

    /// Workflow, agent, and skill attribution supplied by an explicit provenance context.
    pub workflow: Option<WorkflowRecord>,

    /// Environment facts that help a verifier reproduce the output.
    pub environment: Option<EnvironmentRecord>,

    /// Inputs used to produce the asset.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<IoRecord>,

    /// Outputs produced by the same operation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<IoRecord>,

    /// Compact projection of Stencila `ProvenanceCount` values.
    pub provenance_summary: Option<ProvenanceSummaryRecord>,

    /// Reproducibility status and comparison details known at signing time.
    pub verification: VerificationRecord,

    /// Redactions applied while projecting private Stencila state into the assertion.
    pub privacy: PrivacyRecord,

    /// Forward-compatibility slot for future fields.
    #[serde(default, flatten)]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Producer metadata embedded in the assertion.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProducerRecord {
    /// Producer name.
    pub name: String,

    /// Producer version.
    pub version: String,

    /// Stencila Schema version used while producing the asset.
    pub schema_version: Option<String>,

    /// Codec used to encode or export the signed asset.
    pub codec: Option<String>,

    /// Renderer or application component that produced the asset bytes.
    pub renderer: Option<String>,
}

/// Facts about the signed asset.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssetRecord {
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

/// Facts about the Stencila node or work represented by the signed asset.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRecord {
    /// Stencila Schema node type, such as `CodeChunk`, `Figure`, or `Article`.
    pub node_type: String,

    /// Stable Stencila node identifier, when available.
    pub node_id: Option<String>,

    /// Path to the node within the document tree.
    pub node_path: Option<String>,

    /// Stencila label type, such as `FigureLabel` or `TableLabel`.
    pub label_type: Option<String>,

    /// Stencila label for the node, such as `fig:example`.
    pub label: Option<String>,

    /// Human-readable node or work title.
    pub title: Option<String>,

    /// Programming language for executable nodes.
    pub programming_language: Option<String>,

    /// Digests representing the executable node state at signing time.
    pub execution_digest: Option<ExecutionDigestRecord>,
}

/// Digest values corresponding to Stencila `CompilationDigest`.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionDigestRecord {
    /// Digest of execution state that affects generated output.
    pub state_digest: Option<String>,

    /// Digest of semantic content that affects generated output.
    pub semantic_digest: Option<String>,

    /// Digest of dependencies that affect generated output.
    pub dependencies_digest: Option<String>,

    /// Number of stale dependencies known at signing time.
    pub dependencies_stale: Option<u64>,

    /// Number of failed dependencies known at signing time.
    pub dependencies_failed: Option<u64>,
}

/// Source-control facts for the signed output.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceRecord {
    /// Source repository URL or identifier.
    pub repository: Option<String>,

    /// Source commit hash.
    pub commit: Option<String>,

    /// Path to the source document within the repository or project.
    pub path: Option<String>,

    /// Whether uncommitted changes were present.
    pub dirty: Option<bool>,

    /// SHA-256 digest of an unpublished patch, when disclosed.
    pub patch_sha256: Option<String>,

    /// Source tag or release identifier.
    pub tag: Option<String>,
}

/// Execution facts for executable document nodes.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRecord {
    /// Execution status reported by Stencila.
    pub status: Option<String>,

    /// Execution end time.
    pub ended: Option<String>,

    /// Execution duration in milliseconds.
    pub duration_ms: Option<u64>,

    /// Kernel execution counter for the node.
    pub execution_count: Option<i64>,

    /// Kernel used to execute the node.
    pub kernel: Option<KernelRecord>,

    /// Other document nodes this execution depended on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<DependencyRecord>,

    /// Execution messages emitted while producing the asset.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<ExecutionMessageRecord>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KernelRecord {
    /// Kernel name.
    pub name: Option<String>,

    /// Kernel version.
    pub version: Option<String>,

    /// Programming language handled by the kernel.
    pub language: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DependencyRecord {
    /// Depended-on node identifier.
    pub node_id: Option<String>,

    /// Depended-on Stencila node type.
    pub node_type: Option<String>,

    /// Dependency relation, such as input or output.
    pub relation: Option<String>,

    /// Digest of the depended-on node or value.
    pub digest: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionMessageRecord {
    /// Message severity.
    pub level: Option<String>,

    /// Error type or class, for error messages.
    pub error_type: Option<String>,

    /// Human-readable message text.
    pub message: Option<String>,
}

/// Explicit workflow, agent, and skill attribution.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRecord {
    /// Workflow run identifier.
    pub run_id: Option<String>,

    /// Workflow name.
    pub workflow_name: Option<String>,

    /// Digest of the workflow goal or prompt.
    pub goal_digest: Option<String>,

    /// Stencila node identifier associated with the workflow.
    pub node_id: Option<String>,

    /// Conversation or workflow thread identifier.
    pub thread_id: Option<String>,

    /// Produced artifact identifier.
    pub artifact_id: Option<String>,

    /// Agent session identifier.
    pub agent_session_id: Option<String>,

    /// Agent responsible for the workflow.
    pub agent: Option<AgentRecord>,

    /// Definitions loaded by the workflow.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<DefinitionRecord>,

    /// Skills used by the workflow.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_usages: Vec<SkillUsageRecord>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentRecord {
    /// Agent name.
    pub name: Option<String>,

    /// Agent provider.
    pub provider: Option<String>,

    /// Model used by the agent.
    pub model: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionRecord {
    /// Definition kind.
    pub kind: Option<String>,

    /// Definition name.
    pub name: Option<String>,

    /// Source path for the definition.
    pub source_path: Option<String>,

    /// Definition version.
    pub version: Option<String>,

    /// Digest of definition content.
    pub content_hash: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkillUsageRecord {
    /// Skill name.
    pub name: Option<String>,

    /// Source path for the skill.
    pub source_path: Option<String>,

    /// Digest of skill content.
    pub content_hash: Option<String>,

    /// Component that loaded the skill.
    pub loaded_by: Option<String>,

    /// Tool call identifier associated with the skill use.
    pub tool_call_id: Option<String>,

    /// Conversation turn index associated with the skill use.
    pub turn_index: Option<u64>,
}

/// Environment facts selected for publication under the active privacy profile.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentRecord {
    /// Container image used for execution.
    pub container_image: Option<String>,

    /// Operating system name or identifier.
    pub os: Option<String>,

    /// CPU architecture.
    pub architecture: Option<String>,

    /// Runtime versions relevant to reproduction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runtimes: Vec<RuntimeRecord>,

    /// Lockfile digests relevant to reproduction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lockfiles: Vec<FileDigestRecord>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRecord {
    /// Runtime name.
    pub name: Option<String>,

    /// Runtime version.
    pub version: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileDigestRecord {
    /// File path.
    pub path: Option<String>,

    /// SHA-256 digest of the file contents.
    pub sha256: Option<String>,
}

/// Input or output record.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IoRecord {
    /// Input or output kind.
    pub kind: Option<String>,

    /// Input or output name.
    pub name: Option<String>,

    /// Input or output URI.
    pub uri: Option<String>,

    /// IANA media type for the input or output.
    pub media_type: Option<String>,

    /// Digest of the input or output bytes or value.
    pub digest: Option<String>,

    /// Input or output version.
    pub version: Option<String>,

    /// Access level or policy applied to the input or output.
    pub access: Option<String>,

    /// Redaction applied to this input or output.
    pub redaction: Option<String>,

    /// Byte length.
    pub size: Option<u64>,

    /// Width for image, video, or tabular outputs.
    pub width: Option<u64>,

    /// Height for image or video outputs.
    pub height: Option<u64>,

    /// Row count for tabular outputs.
    pub row_count: Option<u64>,

    /// Column count for tabular outputs.
    pub column_count: Option<u64>,

    /// Extension slot for structured IO metadata.
    pub extra: Option<Value>,
}

/// Compact projection of Stencila provenance categories.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceSummaryRecord {
    /// Fraction or percentage attributed to human provenance.
    pub human: Option<f64>,

    /// Fraction or percentage attributed to machine provenance.
    pub machine: Option<f64>,

    /// Fraction or percentage attributed to AI-assisted provenance.
    pub ai_assisted: Option<f64>,

    /// Source of the provenance summary.
    pub source: Option<String>,

    /// Schema version for the provenance summary source.
    pub schema_version: Option<String>,

    /// Per-category provenance counts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<ProvenanceCategoryRecord>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceCategoryRecord {
    /// Provenance category name.
    pub category: String,

    /// Number of characters in the category.
    pub character_count: u64,

    /// Percentage of characters in the category.
    pub character_percent: Option<f64>,
}

/// Reproducibility status and comparison details known when signing.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerificationRecord {
    /// Reproducibility status, such as `not-checked`.
    pub reproducibility_status: String,

    /// Verification policy used.
    pub policy: Option<String>,

    /// Verifier identity.
    pub verified_by: Option<String>,

    /// Verification timestamp.
    pub verified_at: Option<String>,

    /// Structured comparison details.
    pub comparison: Option<Value>,
}

/// Privacy signals and redactions applied while building the assertion.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyRecord {
    /// Redactions applied while building the assertion.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redactions: Vec<RedactionRecord>,

    /// Whether disclosed fields contain personal data.
    pub contains_personal_data: bool,

    /// Whether disclosed fields contain secrets.
    pub contains_secrets: bool,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RedactionRecord {
    /// Redacted field path.
    pub field: Option<String>,

    /// Reason the field was redacted.
    pub reason: Option<String>,
}
