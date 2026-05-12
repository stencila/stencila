//! Stencila provenance facts captured before C2PA signing.
//!
//! A snapshot is the boundary between Stencila's document, execution, workflow,
//! authorship, environment, and export layers and this C2PA-facing crate. It
//! intentionally uses simple serializable values rather than generated Stencila
//! Schema types so callers can project only the facts they want to attest under
//! the selected privacy profile.
//!
//! This module is an internal Stencila Rust API. It is not the C2PA wire format
//! and is not what [`crate::PROVENANCE_SCHEMA`] describes. Before signing, a
//! [`ProvenanceSnapshot`] is normalized into a [`crate::ProvenanceAssertion`],
//! and that assertion is the versioned payload serialized into the
//! `org.stencila.provenance` C2PA assertion.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Internal handoff object used to construct a signed Stencila C2PA assertion.
///
/// Other Stencila crates should populate this from document nodes, author roles,
/// execution state, workflow context, environment probes, export reports, and
/// privacy policy. The signing layer then converts it to
/// [`crate::ProvenanceAssertion`].
///
/// This type is intentionally not the stable published assertion schema. It can
/// evolve as Stencila integration points evolve. Compatibility guarantees belong
/// to [`crate::ProvenanceAssertion`] and its schema URL.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProvenanceSnapshot {
    /// Facts about the signed asset.
    pub asset: AssetSnapshot,

    /// Facts about the root Stencila document node that contains the signed node.
    pub root_node: DocumentSnapshot,

    /// Facts about the executable Stencila node that produced the output node.
    pub executed_node: Option<DocumentSnapshot>,

    /// Facts about the Stencila output node represented by the signed asset.
    pub output_node: Option<DocumentSnapshot>,

    /// Activity that generated, exported, or signed the asset.
    pub activity: Option<ActivitySnapshot>,

    /// Software that produced the C2PA claim and Stencila assertion.
    pub producer: Option<ProducerSnapshot>,

    /// Role-bearing authorship and responsibility facts.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributions: Vec<AttributionSnapshot>,

    /// Source-control facts for the document or project.
    pub source: Option<SourceSnapshot>,

    /// Execution facts for executable outputs.
    pub execution: Option<ExecutionSnapshot>,

    /// Workflow, agent, and definition attribution supplied by an explicit provenance context.
    pub workflow: Option<WorkflowSnapshot>,

    /// Environment facts that help a verifier reproduce the output.
    pub environment: Option<EnvironmentSnapshot>,

    /// Projection of C2PA AI disclosure details.
    pub ai_disclosure: Option<AiDisclosureSnapshot>,

    /// Compact projection of Stencila `ProvenanceCount` values.
    pub provenance_summary: Option<ProvenanceSummarySnapshot>,

    /// Reproducibility status and comparison details known at signing time.
    pub reproducibility: Option<ReproducibilitySnapshot>,

    /// Redactions applied while projecting private Stencila state into the assertion.
    pub privacy: Option<PrivacySnapshot>,

    /// Ingredients to declare via standard `c2pa.ingredient.v3` assertions.
    ///
    /// Use this to surface input or component assets (source files, embedded
    /// figures, datasets) so non-Stencila C2PA tooling can see the provenance
    /// graph rather than treating the asset as appearing from nowhere.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ingredients: Vec<IngredientSnapshot>,
}

impl ProvenanceSnapshot {
    /// Create a minimal snapshot for an already-exported asset.
    #[must_use]
    pub fn for_asset(asset: AssetSnapshot) -> Self {
        Self {
            asset,
            root_node: DocumentSnapshot::default_file(),
            reproducibility: Some(ReproducibilitySnapshot::default()),
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
    pub id: Option<String>,
    pub kind: String,
    pub role: Option<String>,
    pub media_type: String,
    pub content_digest: String,
    pub label: Option<String>,
    pub title: Option<String>,
    pub size: Option<u64>,
    pub width: Option<u64>,
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
            role: None,
            media_type: media_type.into(),
            content_digest: digest.into(),
            ..Default::default()
        }
    }
}

impl Default for AssetSnapshot {
    fn default() -> Self {
        Self {
            id: None,
            kind: "asset".to_string(),
            role: None,
            media_type: String::new(),
            content_digest: String::new(),
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
    pub node_type: String,
    pub node_id: Option<String>,
    pub persistent_id: Option<String>,
    pub node_path: Option<String>,
    pub source_range: Option<SourceRangeSnapshot>,
    pub label_type: Option<String>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub programming_language: Option<String>,
    pub content_url: Option<String>,
    pub media_type: Option<String>,
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
            persistent_id: None,
            node_path: None,
            source_range: None,
            label_type: None,
            label: None,
            title: None,
            programming_language: None,
            content_url: None,
            media_type: None,
        }
    }
}

/// Range of a node in the source document.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct SourceRangeSnapshot {
    pub start_line: u64,
    pub start_column: u64,
    pub end_line: u64,
    pub end_column: u64,
}

/// Activity that generated, exported, or signed the asset.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ActivitySnapshot {
    pub id: Option<String>,
    pub kind: Option<String>,
    pub name: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub associated_attribution_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub used_node_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub generated_node_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub used_asset_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub generated_asset_ids: Vec<String>,
}

/// Software that produced the assertion.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProducerSnapshot {
    pub name: Option<String>,
    pub version: Option<String>,
    pub stencila_schema_version: Option<String>,
    pub codec: Option<String>,
    pub renderer: Option<String>,
}

/// Role-bearing authorship and responsibility facts.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct AttributionSnapshot {
    pub id: Option<String>,
    pub agent: AgentSnapshot,
    pub role_name: Option<String>,
    pub format: Option<String>,
    pub last_modified: Option<String>,
    pub scope: Option<String>,
    pub provenance_category: Option<String>,
    pub character_count: Option<u64>,
    pub character_percent: Option<f64>,
}

/// Agent participating in provenance.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct AgentSnapshot {
    pub kind: Option<String>,
    pub name: Option<String>,
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub identifiers: Vec<IdentifierSnapshot>,
    pub provider: Option<String>,
    pub version: Option<String>,
    pub model: Option<String>,
    pub model_identifier: Option<String>,
    pub url: Option<String>,
}

/// Identifier for an agent.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct IdentifierSnapshot {
    pub kind: Option<String>,
    pub value: Option<String>,
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
    pub patch_digest: Option<String>,
    pub tag: Option<String>,
}

/// Execution facts for executable document nodes.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ExecutionSnapshot {
    pub status: Option<String>,
    pub ended_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub digest: Option<ExecutionDigestSnapshot>,
    pub count: Option<i64>,
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

/// Explicit workflow, agent, and definition attribution.
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
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct DefinitionSnapshot {
    pub kind: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub source_path: Option<String>,
    pub version: Option<String>,
    pub content_digest: Option<String>,
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
    pub digest: Option<String>,
}

/// Projection of C2PA AI disclosure details.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct AiDisclosureSnapshot {
    pub model_type: String,
    pub model_name: Option<String>,
    pub model_identifier: Option<String>,
    pub human_oversight_level: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scientific_domains: Vec<String>,
    pub standard_assertion: Option<String>,
}

/// Compact projection of Stencila provenance categories.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProvenanceSummarySnapshot {
    pub basis: Option<String>,
    pub human_percent: Option<f64>,
    pub machine_percent: Option<f64>,
    pub ai_assisted_percent: Option<f64>,
    pub source: Option<String>,
    pub source_version: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<ProvenanceCategorySnapshot>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProvenanceCategorySnapshot {
    pub provenance_category: String,
    pub character_count: u64,
    pub character_percent: Option<f64>,
}

/// Reproducibility status and comparison details known when signing.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ReproducibilitySnapshot {
    pub reproducibility_status: String,
    pub policy: Option<String>,
    pub checked_by: Option<String>,
    pub checked_at: Option<String>,
    pub comparison: Option<Value>,
}

impl Default for ReproducibilitySnapshot {
    fn default() -> Self {
        Self {
            reproducibility_status: "not-checked".to_string(),
            policy: None,
            checked_by: None,
            checked_at: None,
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
    pub personal_data: DisclosureAssessmentSnapshot,
    pub secrets: DisclosureAssessmentSnapshot,
}

/// Assessment of whether a class of sensitive data is present.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct DisclosureAssessmentSnapshot {
    pub status: String,
    pub policy: Option<String>,
    pub assessed_at: Option<String>,
}

impl Default for DisclosureAssessmentSnapshot {
    fn default() -> Self {
        Self {
            status: "not-assessed".to_string(),
            policy: None,
            assessed_at: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct RedactionSnapshot {
    pub field: Option<String>,
    pub reason: Option<String>,
}

/// C2PA ingredient relationship for an [`IngredientSnapshot`].
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IngredientRelationship {
    /// Direct derivation parent (the asset was opened/converted from this).
    ParentOf,
    /// Input to the process that produced the asset (e.g. a source document
    /// whose code was executed).
    #[default]
    InputTo,
    /// Component embedded inside the asset (e.g. a figure inside a document).
    ComponentOf,
}

/// A standalone asset to declare as a `c2pa.ingredient.v3` ingredient.
///
/// Stencila exports rarely have a single "input file" the way image-editing
/// tools do, but they do have source documents, embedded figures, and other
/// inputs that belong in the C2PA provenance graph. The producer maps each
/// snapshot to a c2pa-rs [`c2pa::Ingredient`] and lets the SDK emit the
/// standard ingredient assertion.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct IngredientSnapshot {
    /// Stable local label used to connect C2PA actions to this ingredient.
    ///
    /// The C2PA SDK resolves action `ingredientIds` against ingredient labels
    /// before writing the final hashed URI references. When omitted, the
    /// producer assigns a deterministic label based on ingredient order.
    pub label: Option<String>,

    /// Display title for the ingredient, typically the file name.
    pub title: Option<String>,

    /// IANA media type of the ingredient bytes.
    pub media_type: Option<String>,

    /// Cryptographic digest of the ingredient bytes in `algorithm:hex` form.
    pub content_digest: Option<String>,

    /// Relationship of the ingredient to the signed asset.
    pub relationship: IngredientRelationship,

    /// Informational URI for verifiers to locate the ingredient (e.g. a
    /// repository URL plus path, a DOI, or a pURL).
    pub informational_uri: Option<String>,

    /// Free-form human-readable description of the ingredient.
    pub description: Option<String>,

    /// Path to a file whose C2PA manifest should be linked into this
    /// ingredient assertion, so verifiers can chain provenance from the
    /// signed asset down to its component or input.
    ///
    /// Either:
    ///
    /// - the ingredient's own bytes when the manifest is embedded (e.g. a
    ///   signed `.png`), or
    /// - a `.c2pa` sidecar file when the manifest is detached.
    ///
    /// When absent, the ingredient is recorded without a manifest link and
    /// verifiers report `ingredient.unknownProvenance` for it.
    pub manifest_source: Option<PathBuf>,
}
