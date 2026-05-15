//! Wire schema for the Stencila C2PA provenance assertion.
//!
//! This module is the source of truth for the JSON payload serialized into the
//! `org.stencila.provenance` C2PA assertion. These types intentionally evolve
//! more conservatively than the internal snapshot types in [`crate::snapshot`]:
//! signed C2PA manifests can outlive the Stencila release that produced them,
//! so every field here should earn its place as a durable public contract.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

/// C2PA assertion label for Stencila provenance.
///
/// The label identifies the assertion inside a C2PA manifest. It is separate
/// from the payload schema URL so that compatible payload evolution can keep the
/// same assertion label and existing verifiers can continue to find the
/// Stencila assertion without knowing every later payload shape.
pub const PROVENANCE_LABEL: &str = "org.stencila.provenance";

/// Payload schema URL for the current Stencila provenance assertion.
///
/// This URL is published alongside other Stencila JSON artifacts under `json/`.
/// The URL itself remains versioned so signed manifests point at an immutable
/// schema artifact. The Rust constant is not version-suffixed because callers
/// usually want the schema this build writes today, while the top-level numeric
/// [`ProvenanceAssertion::version`] carries compatibility semantics.
pub const PROVENANCE_SCHEMA: &str =
    "https://stencila.org/stencila-provenance-assertion-v1.schema.json";

/// C2PA provenance assertion payload used by Stencila content credentials.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceAssertion {
    /// Schema URL identifying the payload family.
    ///
    /// The URL gives validators a dereferenceable schema artifact and a stable
    /// human-facing identifier. It is retained even though `version` is also
    /// present because C2PA assertions are often inspected outside Stencila and
    /// a URL is easier for generic tooling to display or archive.
    pub schema: String,

    /// Numeric payload compatibility version.
    ///
    /// This is fixed at `1` for the first public contract. It exists so Stencila
    /// can later mint refined v1 schema URLs for documentation or optional
    /// additions without causing every exact-URL verifier to report an otherwise
    /// compatible payload as unknown.
    pub version: u16,

    /// Software that produced the C2PA claim and this Stencila assertion.
    ///
    /// This is distinct from `activities.associatedAttributionIds` and `attributions`.
    /// The producer is the claim generator in the C2PA sense, while attributions
    /// describe who or what is credited with creating, generating, verifying, or
    /// accepting the represented content.
    pub producer: ProducerRecord,

    /// Root Stencila document node containing the signed node.
    ///
    /// This gives Stencila-aware consumers a bridge back to the document model
    /// without embedding the full root node JSON. For a root document manifest,
    /// this is also the node exported to the signed asset.
    pub root_node: NodeRecord,

    /// Stencila node that was executed to produce `outputNode`.
    ///
    /// Per-output manifests use this field when the signed asset came from an
    /// executable node, such as a `CodeChunk` that generated an image. It is
    /// omitted for plain document exports and manually signed standalone files.
    pub executed_node: Option<NodeRecord>,

    /// Stencila output node represented by the signed asset.
    ///
    /// For generated media, this is the node in `executedNode.outputs` whose
    /// bytes were exported and signed, such as an `ImageObject`. Keeping this
    /// separate from `asset` matters because the node is Stencila document
    /// structure, while `asset` is the signed byte rendition.
    pub output_node: Option<NodeRecord>,

    /// The source asset entity represented by the signed C2PA manifest.
    ///
    /// This record describes the asset bytes before Content Credentials are added
    /// or normalized by the C2PA SDK. The C2PA hard-binding assertions remain the
    /// authoritative check for the final signed asset bytes. Keeping this record
    /// separate from `outputNode` matters because the same Stencila node may be
    /// exported to several assets, and an asset can be a rendition of a larger
    /// document rather than a node itself.
    pub asset: AssetRecord,

    /// Activities that generated, exported, or signed the asset.
    ///
    /// Provenance needs explicit operations, not just a pile of facts. Records
    /// are ordered from earliest to latest, so a rendered executable document can
    /// record execution before export.
    ///
    /// When an operation is meaningful outside Stencila, producers should also
    /// record it in
    /// [`c2pa.actions.v2`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_actions).
    /// This field then provides the Stencila activity type, timing, and local
    /// relationships needed for reproducibility.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub activities: Vec<ActivityRecord>,

    /// Role-bearing attribution projected from Stencila authorship.
    ///
    /// Stencila documents use `Author` and `AuthorRole` to record human,
    /// organizational, and software contributions. This vector preserves that
    /// concept in a compact form so v1 can answer both "who authored this?" and
    /// "what role did this person or system play?" without later adding a
    /// parallel authorship model.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributions: Vec<AttributionRecord>,

    /// Source-control facts for the document or project.
    ///
    /// Source state is useful for reproducibility and review, but is kept in an
    /// optional record because many exports are not produced from a public or
    /// even local version-control checkout.
    pub source: Option<SourceRecord>,

    /// Execution facts for executable outputs.
    ///
    /// This captures Stencila-specific execution state for code-backed content.
    /// It remains separate from `activity` because not every provenance activity
    /// is a code execution, and one workflow activity may include execution plus
    /// rendering, verification, or signing steps.
    pub execution: Option<ExecutionRecord>,

    /// Workflow, agent, and definition facts supplied by an explicit provenance context.
    ///
    /// Workflow details are optional and may be privacy-sensitive. When present,
    /// they explain the higher-level orchestration around the activity, such as
    /// which workspace workflow run, definition snapshot, or artifact contributed.
    pub workflow: Option<WorkflowRecord>,

    /// Environment facts that help a verifier reproduce the output.
    ///
    /// These are selected under a privacy policy because host and runtime
    /// details can reveal private infrastructure. The record focuses on durable,
    /// reproducibility-relevant facts such as container images, runtimes, and
    /// lockfile digests.
    pub environment: Option<EnvironmentRecord>,

    /// Stencila projection of standard C2PA AI disclosure concepts.
    ///
    /// This field is not a replacement for the standard `c2pa.ai-disclosure`
    /// assertion. It exists so Stencila-specific provenance can cross-reference
    /// model and human-oversight details in the same payload while producers are
    /// still encouraged to emit the standard assertion when model use is
    /// disclosed.
    ///
    /// When both are present, `standardAssertion` can identify the corresponding
    /// [`c2pa.ai-disclosure`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_ai_disclosure)
    /// assertion, while Stencila-specific role and content attribution remains
    /// in `attributions` and `provenance`.
    pub ai_disclosure: Option<AiDisclosureRecord>,

    /// Compact projection of Stencila `ProvenanceCount` values.
    ///
    /// This is a summary, not a substitute for attributions. It answers "how
    /// much of the represented content was human written, machine written, or
    /// reviewed?" using Stencila's stable provenance categories.
    pub provenance: Option<ProvenanceRecord>,

    /// Reproducibility status and comparison details known at signing time.
    ///
    /// Reproducibility is included even when no check was run so
    /// consumers can distinguish "not checked" from "checked and failed" rather
    /// than treating absence as a hidden result.
    pub reproducibility: ReproducibilityRecord,

    /// Privacy policy results and redactions applied while building the assertion.
    ///
    /// C2PA provenance is opt-in and can carry sensitive data. This record makes
    /// the projection policy explicit and avoids over-claiming with bare booleans
    /// such as "contains no secrets" unless an assessment actually ran.
    pub privacy: PrivacyRecord,

    /// Forward-compatibility slot for future top-level fields.
    ///
    /// Stencila preserves unknown fields at every record level so compatible v1
    /// additions can survive a read and write cycle through an older build.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Producer metadata embedded in the assertion.
///
/// This describes the Stencila software component acting as C2PA claim
/// generator. It is intentionally narrower than `AgentRecord`: a producer is
/// about the mechanism that made the manifest, not about authorship credit.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProducerRecord {
    /// Producer name.
    ///
    /// This is usually `Stencila`. It is required so a generic manifest consumer
    /// can display who generated the custom assertion even if it ignores the
    /// nested Stencila-specific records.
    pub name: String,

    /// Producer software version.
    ///
    /// Version is required because signed payloads are immutable. When a schema
    /// interpretation or privacy projection bug is discovered later, the
    /// producer version is the quickest way to scope affected manifests.
    pub version: String,

    /// Stencila Schema version used while producing the asset.
    ///
    /// This is separate from the assertion payload `version`. It records the
    /// document node vocabulary that supplied values such as `nodeType`,
    /// `AuthorRoleName`, and `ProvenanceCategory`.
    pub stencila_schema_version: Option<String>,

    /// Codec used to encode or export the signed asset.
    ///
    /// The codec explains the transformation from Stencila document state to
    /// bytes. It is optional because some manually signed assets have no Stencila
    /// codec involved.
    pub codec: Option<String>,

    /// Renderer or application component that produced the asset bytes.
    ///
    /// Rendering can be independent of the codec, for example a CLI command,
    /// web component, or browser engine. Recording it helps reproduce visual
    /// outputs without overloading `producer.name`.
    pub renderer: Option<String>,

    /// Forward-compatibility slot for future producer metadata.
    ///
    /// Producer details are likely to grow as signing moves into more export
    /// paths. Unknown fields are preserved so older Stencila builds do not erase
    /// that context.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Facts about the source asset entity.
///
/// The asset record is deliberately byte-oriented, but it is not a duplicate of
/// C2PA's own hard-binding assertions. It stores media type, size, dimensions,
/// and a digest of the pre-credential content so Stencila-aware consumers can
/// identify the source rendition without embedding a full Stencila node.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AssetRecord {
    /// Optional asset identifier used by activity references.
    ///
    /// The digest is the cryptographic identity, but a short ID is useful when
    /// `activities.generatedAssetIds` or external reports need to refer to this
    /// asset without repeating a long digest.
    pub id: Option<String>,

    /// Stencila or C2PA-facing asset type.
    ///
    /// The initial vocabulary is `asset`, `image`, `figure`, `table`, `dataset`,
    /// and `document`; reverse-DNS extension values are allowed. The value is a
    /// broad class for UI and policy decisions, not a replacement for
    /// `mediaType`.
    pub asset_type: String,

    /// Asset role in the Stencila export context.
    ///
    /// The role captures why this asset exists, for example `document-export`,
    /// `figure`, `table-image`, or `computational-output`. It complements
    /// `assetType`, which remains the broad media or entity class.
    pub role: Option<String>,

    /// IANA media type for the asset bytes.
    ///
    /// Media type is required because C2PA validators and reproducibility tools
    /// need to know how the bytes should be interpreted independently of file
    /// extension or URL.
    pub media_type: String,

    /// Digest of the pre-credential asset bytes.
    ///
    /// The value should use `algorithm:hex` form, for example `sha256:...`.
    /// Keeping the algorithm in the value avoids baking `sha256` into every
    /// field name and leaves room for future digest algorithms. This digest is
    /// distinct from the C2PA hard binding, which validates the final signed asset
    /// bytes and detects post-signing tampering.
    pub content_digest: String,

    /// Stencila label associated with the asset.
    ///
    /// Labels such as `fig:example` are how authors and readers often refer to
    /// outputs in a document. They are optional because not every signed asset is
    /// a labelled Stencila node.
    pub label: Option<String>,

    /// Human-readable title for the asset.
    ///
    /// This is display metadata for reviewers. It is optional and should not be
    /// treated as a stable identifier because titles can change independently of
    /// the underlying bytes.
    pub title: Option<String>,

    /// Human-readable description for the asset.
    ///
    /// For generated media this may carry the full caption even when `title` is
    /// shortened for display in C2PA tooling.
    pub description: Option<String>,

    /// Asset byte length before signing.
    ///
    /// Size is a cheap consistency check and useful in audit output. It is
    /// optional because some producers may stream content without retaining a
    /// length at projection time.
    pub size: Option<u64>,

    /// Width for image or video assets.
    ///
    /// Dimensions help consumers understand the signed rendition without opening
    /// the asset. Width is optional because it only applies to some media types.
    pub width: Option<u64>,

    /// Height for image or video assets.
    ///
    /// Dimensions are recorded as unsigned integers in rendered pixel units
    /// unless the media type defines a different native unit.
    pub height: Option<u64>,

    /// Forward-compatibility slot for future asset metadata.
    ///
    /// Asset-specific fields are especially likely to grow for audio, video,
    /// compound documents, and structured text, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Facts about a Stencila node related to the signed asset.
///
/// This record anchors C2PA bytes back to Stencila document structure while
/// staying compact enough for manifests. It stores stable node identity and
/// selected public metadata rather than embedding private source content.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeRecord {
    /// Stencila Schema node type.
    ///
    /// Values such as `CodeChunk`, `Figure`, `Table`, `Article`, or `File` let
    /// Stencila-aware consumers recover the kind of work represented by the
    /// asset without depending on media type heuristics. Values intentionally
    /// use Stencila Schema's `PascalCase` node type convention.
    pub node_type: String,

    /// Stable Stencila node identifier, when available.
    ///
    /// This is a deterministic structural identifier derived from the node's
    /// content, label, or position in the stabilized document tree (e.g.
    /// `content-1-content-3`). It is stable across re-renders of the same
    /// source document and lets later tooling correlate credentials with the
    /// source document tree. It is optional because standalone asset signing
    /// and imported files may not have a stable Stencila node ID.
    pub node_id: Option<String>,

    /// Author-supplied persistent identifier from the Stencila Schema `id`
    /// field.
    ///
    /// This is the DOM-style identifier the document author wrote, separate
    /// from the structural `nodeId`. When the author has set an `id` (for
    /// example `id: "fig-plot"` on a Figure), it is recorded here unchanged so
    /// verifiers can locate the node by its author-given name without needing
    /// to know Stencila's structural identifier scheme.
    pub persistent_id: Option<String>,

    /// Path to the node within the document tree.
    ///
    /// A path is useful when IDs are absent or when reviewers need to locate the
    /// represented node in a specific document snapshot. It is optional because
    /// paths can reveal document structure and can be unstable across edits.
    pub node_path: Option<String>,

    /// Range of the node in the source document.
    ///
    /// Positions are 1-based UTF-8 line and column coordinates, with an
    /// exclusive end position. The range covers the whole serialized node in
    /// the source document, not only one of its properties.
    pub source_range: Option<SourceRangeRecord>,

    /// Stencila label type for labelled nodes.
    ///
    /// Label type distinguishes figure, table, equation, and other label
    /// namespaces without parsing the label string itself.
    pub label_type: Option<String>,

    /// Stencila label for the node.
    ///
    /// This repeats the asset label when the signed asset is a direct rendition
    /// of a labelled node. Keeping it here allows document-level verification
    /// even when the asset record is used for an unlabelled file rendition.
    pub label: Option<String>,

    /// Human-readable node or work title.
    ///
    /// Titles improve audit readability but remain optional because many
    /// executable nodes and generated outputs are intentionally untitled.
    pub title: Option<String>,

    /// Programming language for executable nodes.
    ///
    /// Language belongs on the document node, not only the kernel, because it is
    /// part of the authored source semantics that affect execution and review.
    pub programming_language: Option<String>,

    /// URL or path for media-like nodes.
    ///
    /// This is useful for output nodes such as `ImageObject`, `AudioObject`,
    /// `MediaObject`, and `VideoObject`, but is optional because many Stencila
    /// nodes are not byte-backed media references.
    pub content_url: Option<String>,

    /// IANA media type for media-like nodes.
    ///
    /// This is node metadata. The signed bytes remain described by
    /// `asset.mediaType`, which may differ for alternate renditions.
    pub media_type: Option<String>,

    /// Forward-compatibility slot for future document metadata.
    ///
    /// Stencila Schema will evolve faster than this assertion. Unknown fields are
    /// kept so optional projections from newer node types are not erased.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Range coordinates in the source document.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceRangeRecord {
    /// 1-based index of the first line in the source range.
    #[schemars(range(min = 1))]
    pub start_line: u64,

    /// 1-based UTF-8 column index of the start of the source range.
    #[schemars(range(min = 1))]
    pub start_column: u64,

    /// 1-based index of the line containing the exclusive end position.
    #[schemars(range(min = 1))]
    pub end_line: u64,

    /// 1-based UTF-8 column index of the exclusive end position.
    #[schemars(range(min = 1))]
    pub end_column: u64,
}

/// Digest values corresponding to Stencila `CompilationDigest`.
///
/// Stencila distinguishes state, semantic content, and dependency digests so a
/// verifier can understand why an output might be stale without seeing the full
/// source. This record preserves that distinction in the public assertion.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionDigestsRecord {
    /// Digest of execution state that affects generated output.
    ///
    /// This usually covers code, parameter values, and other state needed to
    /// decide whether an executable node should be rerun.
    pub state: Option<String>,

    /// Digest of semantic content that affects generated output.
    ///
    /// Semantic digests let Stencila distinguish meaningful source changes from
    /// formatting or metadata churn.
    pub semantic: Option<String>,

    /// Digest of dependencies that affect generated output.
    ///
    /// This summarizes upstream nodes or values used by the executable node. It
    /// is separate from `dependencies` because the digest is compact and stable
    /// even when individual dependencies are redacted.
    pub dependencies: Option<String>,

    /// Number of stale dependencies known at signing time.
    ///
    /// A nonzero value warns verifiers that the output may not reflect current
    /// upstream state even if the signed asset itself is valid.
    pub dependencies_stale: Option<u64>,

    /// Number of failed dependencies known at signing time.
    ///
    /// Failed dependencies are recorded separately from stale dependencies
    /// because they indicate an execution failure path rather than merely an
    /// out-of-date one.
    pub dependencies_failed: Option<u64>,

    /// Forward-compatibility slot for future digest components.
    ///
    /// The digest model may grow as Stencila tracks more kinds of dependency and
    /// semantic state, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Activity that generated, exported, or signed the asset.
///
/// This is the provenance relation hub. It intentionally uses IDs to refer to
/// attributions, Stencila nodes, and byte assets so the assertion can remain
/// compact while still expressing the important "activity used/generated/was
/// associated with" relationships.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRecord {
    /// Optional activity identifier.
    ///
    /// IDs make it possible for workflow logs, verification reports, or later
    /// assertions to refer to this operation. They are optional because manual
    /// signing often has no durable external run ID.
    pub id: Option<String>,

    /// Activity type.
    ///
    /// The initial vocabulary is `sign`, `export`, `execute`, `render`, and
    /// `run`; reverse-DNS extension values are allowed. This field is required
    /// so consumers do not have to infer the operation from whichever optional
    /// detail records happen to be present.
    pub activity_type: String,

    /// Human-readable activity name.
    ///
    /// A name is useful for workflows and UI summaries, but not stable enough to
    /// serve as the activity type or identifier.
    pub name: Option<String>,

    /// Activity start time in RFC 3339 format.
    ///
    /// Start time is optional because some integrations only know when execution
    /// ended or when the C2PA claim was signed.
    pub started_at: Option<String>,

    /// Activity end time in RFC 3339 format.
    ///
    /// End time is often the most useful reproducibility timestamp because it is
    /// when generated outputs became available for signing.
    pub ended_at: Option<String>,

    /// Activity duration in milliseconds.
    ///
    /// Duration is stored as an integer instead of a formatted string so
    /// verifiers can compare timings across runs without parsing display units.
    pub duration_ms: Option<u64>,

    /// Attribution IDs associated with the activity.
    ///
    /// These IDs point at entries in `attributions`. The vector records the PROV
    /// style association without duplicating full attribution records in the
    /// activity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub associated_attribution_ids: Vec<String>,

    /// Stencila node IDs used by the activity.
    ///
    /// These IDs point at `rootNode`, `executedNode`, `outputNode`, or other
    /// Stencila nodes known to the producer. They express use relationships
    /// without embedding the full source node content.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub used_node_ids: Vec<String>,

    /// Stencila node IDs generated by the activity.
    ///
    /// For executable output manifests, the execution activity usually records
    /// `outputNode.nodeId` here.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generated_node_ids: Vec<String>,

    /// Asset IDs used by the activity.
    ///
    /// These IDs point at byte assets rather than Stencila nodes. They are
    /// optional because many activities only operate over document nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub used_asset_ids: Vec<String>,

    /// Asset IDs generated by the activity.
    ///
    /// For export or signing activities, this usually points at `asset.id`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generated_asset_ids: Vec<String>,

    /// Forward-compatibility slot for future activity metadata.
    ///
    /// Activity modeling is expected to grow as workflows become richer, so
    /// unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Role-bearing attribution for an agent.
///
/// This is the public, compact equivalent of Stencila `AuthorRole`. It records
/// the contributing agent, the role they played, optional Stencila provenance
/// counts, and the scope of the attribution.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributionRecord {
    /// Optional attribution identifier.
    ///
    /// Activity records refer to associated agents through this ID. It is
    /// optional so simple assertions can still list attributions without
    /// inventing local identifiers.
    pub id: Option<String>,

    /// Agent credited or responsible for the role.
    ///
    /// The agent can be a person, organization, software application, model, or
    /// intentionally anonymous thing. Using a common agent record keeps human and
    /// machine authorship in the same Stencila-compatible attribution model.
    pub agent: AgentRecord,

    /// Stencila `AuthorRoleName` or compatible role value.
    ///
    /// Known values include `Writer`, `Verifier`, `Instructor`, `Generator`, and
    /// `Executor`. The field is optional because flat Stencila `Author` values
    /// and external metadata can identify an author without a known role.
    pub role_name: Option<String>,

    /// Format in which the role was performed.
    ///
    /// This mirrors `AuthorRole.format`, for example Markdown, Python, or HTML.
    /// It helps distinguish writing source code from editing rendered output.
    pub format: Option<String>,

    /// Most recent modification time by this agent in this role.
    ///
    /// The timestamp should be RFC 3339. It is optional because privacy profiles
    /// may strip timestamps and because not all author metadata tracks last
    /// modification.
    pub last_modified: Option<String>,

    /// Scope of the attribution.
    ///
    /// Suggested values are `document`, `asset`, `activity`, `input`, and
    /// `output`. The scope prevents a workflow executor from being mistaken for
    /// a bibliographic author of the whole work.
    pub scope: Option<String>,

    /// Stencila provenance category associated with this attribution.
    ///
    /// Values come from `ProvenanceCategory`, such as `Hw`, `Mw`, or `MwHeHv`.
    /// The category is optional because authorship roles and provenance counts
    /// are related but not identical concepts.
    pub provenance_category: Option<String>,

    /// Number of characters attributed to this agent and role.
    ///
    /// Character counts are optional because they only apply to text-like
    /// content and may be redacted for privacy or size reasons.
    pub character_count: Option<u64>,

    /// Percentage of counted content attributed to this agent and role.
    ///
    /// This is a 0-100 percentage, matching Stencila `ProvenanceCount` semantics.
    /// It is optional because counts may be available without a computed percent.
    pub character_percent: Option<f64>,

    /// Forward-compatibility slot for future attribution metadata.
    ///
    /// Author and agent metadata is likely to gain richer identity and role
    /// details, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Agent participating in provenance.
///
/// The same record is used for human, organizational, software, and model
/// agents. That keeps the schema aligned with Stencila `Author` and `AuthorRole`
/// while also mapping cleanly to C2PA's broader notion of actors.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentRecord {
    /// Agent type.
    ///
    /// Suggested values are `person`, `organization`, `softwareApplication`,
    /// `model`, and `thing`. It is optional so redacted or legacy agents can
    /// still be represented by name or identifier alone.
    pub agent_type: Option<String>,

    /// Agent name.
    ///
    /// Names are display metadata and may be pseudonymous or redacted. Stable
    /// identity should be represented in `identifiers` where possible.
    pub name: Option<String>,

    /// Stable local or external agent identifier.
    ///
    /// This field is for a primary identifier that other records may reference.
    /// Additional identity schemes belong in `identifiers`.
    pub id: Option<String>,

    /// Structured identifiers for the agent.
    ///
    /// This supports ORCID for people, ROR for organizations, package URLs for
    /// software, and model identifiers without adding a top-level field for each
    /// identity scheme.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identifiers: Vec<IdentifierRecord>,

    /// Organization, service, or model provider.
    ///
    /// Provider is optional because not all agents are mediated by a provider,
    /// but it is useful for AI and remote execution agents where model names are
    /// not globally unique.
    pub provider: Option<String>,

    /// Software or model version.
    ///
    /// Version is kept on the agent because generator and executor software can
    /// be credited as authors independently of the Stencila producer version.
    pub version: Option<String>,

    /// Model name used by an AI agent.
    ///
    /// This remains optional and display-oriented. Durable model identity should
    /// use `modelIdentifier` or a typed identifier.
    pub model: Option<String>,

    /// Durable model identifier, URI, or package URL.
    ///
    /// Keeping this separate from `model` allows a readable name and a stable
    /// machine identifier to coexist.
    pub model_identifier: Option<String>,

    /// Agent URL.
    ///
    /// URLs are useful for software and organization records but optional
    /// because they can expose private infrastructure or become stale.
    pub url: Option<String>,

    /// Forward-compatibility slot for future agent metadata.
    ///
    /// Identity standards change over time. Unknown agent fields are preserved so
    /// newer identifiers are not lost by older verifiers.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Identifier for an agent or other named entity.
///
/// A small typed identifier record is more future-proof than adding separate
/// optional fields for every identity scheme that Stencila may later support.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IdentifierRecord {
    /// Identifier type or scheme.
    ///
    /// Examples include `orcid`, `ror`, `url`, `purl`, `doi`, and `modelId`.
    /// Consumers should treat unknown types as opaque labels.
    pub identifier_type: Option<String>,

    /// Identifier value.
    ///
    /// The value is a string because identity schemes differ in syntax and some
    /// use URIs while others use compact IDs.
    pub value: Option<String>,

    /// Forward-compatibility slot for future identifier metadata.
    ///
    /// Identifier records may later need verification state, issuer details, or
    /// proof material, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Source-control facts for the signed output.
///
/// Source information helps reviewers and automated systems locate the authored
/// document state that led to the signed asset. Each field is optional because
/// privacy policy or offline workflows may disclose only a subset.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceRecord {
    /// Source repository URL or identifier.
    ///
    /// This may be a public URL, an internal repository identifier, or a redacted
    /// stable label. It is optional because repository names can be sensitive.
    pub repository: Option<String>,

    /// Source commit hash or revision identifier.
    ///
    /// The field name is intentionally generic enough for Git and non-Git
    /// systems, while the documentation calls out the common commit use case.
    pub commit: Option<String>,

    /// Path to the source document within the repository or project.
    ///
    /// Paths are useful for reproducibility but can leak project structure, so
    /// they are optional and may be redacted under the privacy policy.
    pub path: Option<String>,

    /// Whether uncommitted changes were present.
    ///
    /// This boolean is optional rather than defaulting to false because "not
    /// assessed" and "clean" are different provenance states.
    pub dirty: Option<bool>,

    /// Digest of an unpublished patch, when disclosed.
    ///
    /// The value should use `algorithm:hex` form. A patch digest lets a signer
    /// attest that local changes existed without embedding the patch itself.
    pub patch_digest: Option<String>,

    /// Source tag or release identifier.
    ///
    /// Tags help link the signed asset to a release even when a full commit hash
    /// is not appropriate for disclosure.
    pub tag: Option<String>,

    /// Forward-compatibility slot for future source metadata.
    ///
    /// Source-control systems vary widely, so unknown fields are preserved for
    /// future providers and repository receipt integrations.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Execution facts for executable document nodes.
///
/// This record mirrors Stencila execution state. It is narrower than `activity`
/// and should be present when the represented asset depends on executing code,
/// a prompt, a query, or another executable document node.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRecord {
    /// Execution status reported by Stencila.
    ///
    /// Values should come from Stencila `ExecutionStatus` when possible, for
    /// example `Succeeded`, `Failed`, or `Skipped`. Extension strings are allowed
    /// for future statuses.
    pub status: Option<String>,

    /// Execution end time in RFC 3339 format.
    ///
    /// The field is named `endedAt` to match the activity timing vocabulary.
    /// The legacy `ended` name is accepted for unpublished pre-v1 payloads.
    pub ended_at: Option<String>,

    /// Execution duration in milliseconds.
    ///
    /// Milliseconds give enough precision for user-facing diagnostics without
    /// encoding a language-specific duration format.
    pub duration_ms: Option<u64>,

    /// Digests representing executable node state at signing time.
    ///
    /// These compactly attest the state that Stencila considered relevant to
    /// generated output, without disclosing the source code or dependency values
    /// themselves. The field is named `digests` because it is already nested
    /// under `execution`.
    pub digests: Option<ExecutionDigestsRecord>,

    /// Kernel execution counter for the node.
    ///
    /// This preserves notebook-style execution context for reviewers. It is
    /// optional because many kernels or workflows do not expose a counter.
    pub count: Option<i64>,

    /// Kernel used to execute the node.
    ///
    /// Kernel identity is separate from environment runtime versions because a
    /// named kernel can wrap a language runtime, container, or remote service.
    pub kernel: Option<KernelRecord>,

    /// Other document nodes or values this execution depended on.
    ///
    /// Dependencies explain why an output should be rerun when upstream state
    /// changes. They can be redacted or summarized independently of aggregate
    /// digest values on this execution record.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<DependencyRecord>,

    /// Execution messages emitted while producing the asset.
    ///
    /// Messages are kept compact because manifests should not become log files.
    /// They are included to preserve warnings or errors that materially affect
    /// trust in the output.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<ExecutionMessageRecord>,

    /// Forward-compatibility slot for future execution metadata.
    ///
    /// Execution backends vary, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Kernel used to execute a Stencila node.
///
/// Kernels are separated from runtimes because the same language runtime can be
/// exposed through different kernels with different execution semantics.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KernelRecord {
    /// Kernel name.
    ///
    /// The name should be stable enough for a human to identify the execution
    /// backend, for example `python`, `r`, or a provider-specific kernel name.
    pub name: Option<String>,

    /// Kernel version.
    ///
    /// Version is optional because some kernels are wrappers around runtimes
    /// whose exact version is better captured in `environment.runtimes`.
    pub version: Option<String>,

    /// Programming language handled by the kernel.
    ///
    /// Language helps verifiers select reproduction tooling even when the kernel
    /// name is provider-specific.
    pub language: Option<String>,

    /// Forward-compatibility slot for future kernel metadata.
    ///
    /// Remote and multi-language kernels may need additional fields later.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Dependency of an execution.
///
/// Dependencies represent the specific upstream nodes, values, or artifacts that
/// Stencila knew the execution used. They complement the aggregate dependency
/// digest on `ExecutionRecord`.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DependencyRecord {
    /// Depended-on node identifier.
    ///
    /// Node IDs let verifiers connect dependency facts back to a Stencila
    /// document tree when that tree is available.
    pub node_id: Option<String>,

    /// Depended-on Stencila node type.
    ///
    /// The node type helps consumers understand whether the dependency was code,
    /// data, prose, a parameter, or another output. When it names a Stencila
    /// Schema node type, use Stencila Schema's `PascalCase` convention.
    pub node_type: Option<String>,

    /// Dependency relation.
    ///
    /// Suggested values are `input`, `output`, `state`, and `parameter`; custom
    /// values should use a reverse-DNS prefix when they are not Stencila terms.
    pub relation: Option<String>,

    /// Digest of the depended-on node or value.
    ///
    /// The value should use `algorithm:hex` form. It is optional because some
    /// dependencies are intentionally disclosed only by identity or relation.
    pub digest: Option<String>,

    /// Forward-compatibility slot for future dependency metadata.
    ///
    /// More detailed dependency graphs can be added later without losing unknown
    /// nested fields.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Message emitted during execution.
///
/// Messages provide trust-relevant warnings and errors while avoiding full log
/// capture. They should be filtered under the privacy policy before signing.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionMessageRecord {
    /// Message severity.
    ///
    /// Values should align with Stencila `MessageLevel`, for example `Info`,
    /// `Warning`, or `Error`.
    pub level: Option<String>,

    /// Error type or class, for error messages.
    ///
    /// This supports automated triage without requiring consumers to parse the
    /// human-readable message text.
    pub error_type: Option<String>,

    /// Human-readable message text.
    ///
    /// Message text is optional and should be redacted when it contains source
    /// code, secrets, private paths, or other sensitive details.
    pub message: Option<String>,

    /// Forward-compatibility slot for future message metadata.
    ///
    /// Future structured diagnostics can be added without dropping unknown
    /// fields.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Explicit workflow, agent, and definition context.
///
/// Workflows describe orchestration around the activity. The fields align with
/// the Stencila workspace database: `runId` identifies `workflow_runs.run_id`,
/// `nodeId` identifies `workflow_nodes.node_id` within that run, `artifactId`
/// identifies `workflow_artifacts.artifact_id`, and definition `contentDigest`
/// values identify `workflow_definition_snapshots.content_hash`.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRecord {
    /// Workflow run identifier.
    ///
    /// A run ID lets Stencila and external audit systems correlate the assertion
    /// with `workflow_runs.run_id` without embedding the full workflow log.
    pub run_id: Option<String>,

    /// Workflow name.
    ///
    /// The name is display metadata and may be redacted or omitted for private
    /// workflows.
    pub workflow_name: Option<String>,

    /// Digest of the workflow goal or prompt.
    ///
    /// The digest gives evidence that a particular private goal existed without
    /// disclosing the prompt text.
    pub goal_digest: Option<String>,

    /// Stencila node identifier associated with the workflow.
    ///
    /// This may differ from `rootNode.nodeId`, `executedNode.nodeId`, or
    /// `outputNode.nodeId` when a workflow operates over a broader scope than
    /// the signed asset.
    pub node_id: Option<String>,

    /// Conversation or workflow thread identifier.
    ///
    /// Thread IDs help relate multi-turn agent work while remaining optional for
    /// privacy and for non-conversational workflows.
    pub thread_id: Option<String>,

    /// Produced artifact identifier.
    ///
    /// This is a workflow-system artifact ID, not necessarily the C2PA asset ID.
    /// It is optional because artifacts may be transient or private.
    pub artifact_id: Option<String>,

    /// Agent session identifier.
    ///
    /// Session IDs are useful for audit trails but can be sensitive, so they are
    /// optional and subject to the privacy policy.
    pub agent_session_id: Option<String>,

    /// Agent responsible for the workflow.
    ///
    /// This records the orchestrating agent, while `attributions` records the
    /// role-bearing authorship or responsibility credited to agents.
    pub agent: Option<AgentRecord>,

    /// Definitions loaded by the workflow.
    ///
    /// Definitions are recorded by metadata and digest so their content can stay
    /// private while still being linked to `workflow_definition_snapshots`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<DefinitionRecord>,

    /// Forward-compatibility slot for future workflow metadata.
    ///
    /// Workflow engines evolve quickly, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Definition loaded by a workflow.
///
/// Definitions represent workflow, agent, skill, policy, or other reusable
/// resources that may affect generated outputs. The `definitionType`, `name`, `role`,
/// and `contentDigest` fields are sufficient to link a record back to the
/// Stencila workspace definition tables when `runId` is also disclosed.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionRecord {
    /// Definition type.
    ///
    /// Type distinguishes workspace definition snapshots such as `workflow`,
    /// `agent`, and `skill` without requiring a separate record for each class.
    pub definition_type: Option<String>,

    /// Definition name.
    ///
    /// Names help audit readability but are optional because they can reveal
    /// private project structure.
    pub name: Option<String>,

    /// Role this definition played in the workflow run.
    ///
    /// Role corresponds to `workflow_run_definitions.role`. It lets consumers
    /// disambiguate how a content-addressed definition snapshot contributed to
    /// the run, for example as a workflow, agent, skill, or policy.
    pub role: Option<String>,

    /// Source path for the definition.
    ///
    /// Paths are useful for reproduction but often private, so this field should
    /// be set only when the active privacy policy permits it.
    pub source_path: Option<String>,

    /// Definition version.
    ///
    /// Version complements the content digest when definitions are distributed
    /// as named packages or managed resources.
    pub version: Option<String>,

    /// Digest of definition content.
    ///
    /// The value should use `algorithm:hex` form. It is preferred over raw
    /// content so private definitions can still be attested.
    pub content_digest: Option<String>,

    /// Forward-compatibility slot for future definition metadata.
    ///
    /// Definition systems may add registry, namespace, or signature details.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Environment facts selected for publication under the active privacy profile.
///
/// The environment record captures reproducibility context without attempting to
/// be a full software bill of materials. It keeps the assertion compact and
/// privacy-aware while still surfacing the facts most likely to affect reruns.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentRecord {
    /// Container image used for execution.
    ///
    /// A container image is often the strongest environment identity, but it is
    /// optional because many local or hosted executions do not use containers or
    /// cannot disclose image names.
    pub container_image: Option<String>,

    /// Operating system name or identifier.
    ///
    /// OS details are useful for reproduction but can reveal infrastructure, so
    /// they should be disclosed only under an appropriate privacy policy.
    pub os: Option<String>,

    /// CPU architecture.
    ///
    /// Architecture can affect binary packages and numeric results. It is kept
    /// separate from OS because cross-platform containers can mix the two.
    pub architecture: Option<String>,

    /// Runtime versions relevant to reproduction.
    ///
    /// Runtime records are intentionally lightweight and ordered by the producer
    /// so a verifier can display the most important runtimes first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runtimes: Vec<RuntimeRecord>,

    /// Environment manifest digests relevant to reproduction.
    ///
    /// Manifests identify declared dependency and tool requirements, complementing
    /// lockfiles that identify resolved dependency state.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manifests: Vec<FileDigestRecord>,

    /// Lockfile digests relevant to reproduction.
    ///
    /// Digests identify dependency state without embedding potentially large or
    /// private lockfiles in the manifest.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lockfiles: Vec<FileDigestRecord>,

    /// Source repository URL or identifier for the environment context.
    ///
    /// This usually matches the source repository, but is stored with the
    /// environment so the manifest and lockfile paths can be resolved even when
    /// the signed asset does not disclose a source document.
    pub repository: Option<String>,

    /// Source commit hash for the environment context.
    ///
    /// A full commit hash lets verifiers locate immutable manifest and lockfile
    /// contents when the repository URL is public.
    pub commit: Option<String>,

    /// Informational URI for locating the environment context.
    ///
    /// This should point to immutable, human-browsable environment context such
    /// as a repository tree at a full commit hash.
    pub informational_uri: Option<String>,

    /// Forward-compatibility slot for future environment metadata.
    ///
    /// Environment provenance may later include hardware, accelerators, locale,
    /// or package-manager details, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Runtime version relevant to reproduction.
///
/// Runtime records cover language runtimes, package managers, database engines,
/// or other executable environments that materially affect outputs.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRecord {
    /// Runtime name.
    ///
    /// Examples include `python`, `r`, `node`, `quarto`, or a package manager
    /// name. It is optional for redacted or partially known environments.
    pub name: Option<String>,

    /// Runtime version.
    ///
    /// Version is optional because some runtimes are implicit in a container or
    /// unavailable from the execution backend.
    pub version: Option<String>,

    /// Forward-compatibility slot for future runtime metadata.
    ///
    /// Future runtimes may need distribution, channel, or build identifiers.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Digest of a file relevant to reproduction.
///
/// This record is used for lockfiles and other local files where disclosing the
/// path and digest is useful but embedding the full file would be too large or
/// private.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileDigestRecord {
    /// File path.
    ///
    /// Paths are optional because they can reveal private project structure.
    /// When omitted, the digest can still identify the file content.
    pub path: Option<String>,

    /// Digest of the file contents.
    ///
    /// The value should use `algorithm:hex` form. The legacy `sha256` field name
    /// is accepted for unpublished pre-v1 payloads.
    pub digest: Option<String>,

    /// Forward-compatibility slot for future file metadata.
    ///
    /// File records may later include size, media type, or package-manager
    /// semantics, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Stencila projection of C2PA AI disclosure concepts.
///
/// This record keeps Stencila-specific provenance summaries connected to the AI
/// transparency vocabulary used by C2PA. It mirrors the standard field names
/// where they are stable, but remains a local projection. Producers that
/// disclose AI model use should also emit the standard `c2pa.ai-disclosure`
/// assertion when possible.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiDisclosureRecord {
    /// C2PA AI model type.
    ///
    /// This mirrors the standard AI disclosure `modelType` field so Stencila
    /// consumers can reason about the model class without parsing another
    /// assertion.
    pub model_type: String,

    /// Human-readable model name.
    ///
    /// The name is display metadata. Durable identity should use
    /// `modelIdentifier` where available.
    pub model_name: Option<String>,

    /// Unique model identifier, URI, or package URL.
    ///
    /// This mirrors the standard AI disclosure `modelIdentifier` field and is
    /// preferred for reproducibility and compliance checks.
    pub model_identifier: Option<String>,

    /// Content profile for the AI use.
    ///
    /// This mirrors the standard AI disclosure `contentProfile` object for
    /// structured content-context and human-oversight signals.
    pub content_profile: Option<AiContentProfileRecord>,

    /// Scientific domains associated with the AI use.
    ///
    /// Values should follow the C2PA AI disclosure convention, such as arXiv
    /// taxonomy strings, when disclosed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scientific_domain: Vec<String>,

    /// Label or URI of the corresponding standard C2PA assertion.
    ///
    /// This optional pointer prevents the Stencila projection from drifting away
    /// from the standard assertion when both are present.
    pub standard_assertion: Option<String>,

    /// Forward-compatibility slot for future AI disclosure metadata.
    ///
    /// AI transparency fields are evolving quickly, so unknown fields are
    /// preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Content profile for AI disclosure.
///
/// The record mirrors the standard C2PA AI disclosure `contentProfile` object
/// while allowing Stencila to preserve unknown future profile fields.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiContentProfileRecord {
    /// Human oversight level.
    ///
    /// Values should follow C2PA AI disclosure, for example
    /// `fully_autonomous`, `prompt_guided`, or `human_validated`.
    pub human_oversight_level: Option<String>,

    /// Forward-compatibility slot for future content profile metadata.
    ///
    /// C2PA AI disclosure fields are evolving, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Compact projection of Stencila provenance categories.
///
/// The summary is derived from Stencila `ProvenanceCount` values. It uses a
/// single 0-100 percentage convention to avoid the previous ambiguity between
/// fractions and percentages.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceRecord {
    /// Measurement basis for counts and percentages.
    ///
    /// The default Stencila basis is `characters`. The field exists because
    /// future summaries for images, tables, or multimodal outputs may use bytes,
    /// pixels, cells, tokens, or another unit.
    pub basis: Option<String>,

    /// Percentage attributed to human-only provenance.
    ///
    /// The value is a 0-100 percentage, not a fraction. It is optional because
    /// consumers can always recompute it from categories when complete category
    /// counts are disclosed.
    pub human_percent: Option<f64>,

    /// Percentage attributed to machine-only provenance.
    ///
    /// The value is a 0-100 percentage. This usually covers Stencila provenance
    /// categories where machine generation or editing was present without human
    /// editing or verification.
    pub machine_percent: Option<f64>,

    /// Percentage attributed to mixed human and machine provenance.
    ///
    /// This value is a Stencila summary convenience, not a regulatory AI-use
    /// declaration. Use `aiDisclosure` and standard C2PA AI disclosure for model
    /// transparency.
    pub ai_assisted_percent: Option<f64>,

    /// Source of the provenance summary.
    ///
    /// This should usually be `stencila-provenance-counts`. It is optional so
    /// imported or externally computed summaries can identify their source.
    pub source: Option<String>,

    /// Version of the summary source or algorithm.
    ///
    /// This is separate from the assertion payload version because provenance
    /// counting algorithms may change without changing the wire schema.
    pub source_version: Option<String>,

    /// Per-category provenance counts.
    ///
    /// Categories preserve the Stencila `ProvenanceCategory` vocabulary so
    /// detailed consumers are not limited to coarse human/machine rollups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<ProvenanceCategoryRecord>,

    /// Forward-compatibility slot for future summary metadata.
    ///
    /// Additional rollups or basis-specific statistics can be added without
    /// losing unknown fields.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Per-category count for a Stencila provenance category.
///
/// This mirrors `ProvenanceCount` closely enough that a consumer can map it back
/// to Stencila concepts without loading the full generated Stencila Schema.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceCategoryRecord {
    /// Stencila provenance category.
    ///
    /// Values should come from `ProvenanceCategory`, such as `Hw`, `HwMv`,
    /// `MwHe`, or `MwMeMv`. The legacy `category` field name is accepted for
    /// unpublished payloads.
    pub category: String,

    /// Number of counted units in the category.
    ///
    /// The unit is defined by `ProvenanceRecord.basis`, usually
    /// characters. This field is required because percentages alone are not
    /// enough to compare summaries across differently sized works.
    pub character_count: u64,

    /// Percentage of counted content in this category.
    ///
    /// The value is a 0-100 percentage, matching Stencila
    /// `ProvenanceCount.characterPercent`.
    pub character_percent: Option<f64>,

    /// Forward-compatibility slot for future category metadata.
    ///
    /// Future category records may include basis-specific counts or confidence
    /// metadata, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Reproducibility status and comparison details known when signing.
///
/// This record reports what Stencila knew at signing time. It is not the same as
/// C2PA manifest validation, which verifies signatures and asset binding.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReproducibilityRecord {
    /// Reproducibility status.
    ///
    /// Suggested values are `not-checked`, `reproduced`, `changed`, `failed`,
    /// and `not-reproducible`. The field is required so absence never masks an
    /// intentionally skipped reproducibility check.
    pub status: String,

    /// Reproducibility check policy used.
    ///
    /// Policy explains how strict the comparison was, which outputs were
    /// compared, and what tolerances applied. It is optional because initial
    /// signing often records `not-checked`.
    pub policy: Option<String>,

    /// Reproducibility checker identity.
    ///
    /// This can be a person, service, or software identifier. Rich checker
    /// attribution should also be represented in `attributions` when relevant.
    pub checked_by: Option<String>,

    /// Reproducibility check timestamp in RFC 3339 format.
    ///
    /// The timestamp records when reproducibility was checked, not when the C2PA
    /// claim was signed.
    pub checked_at: Option<String>,

    /// Structured comparison details.
    ///
    /// This intentionally remains JSON so comparison tools can include
    /// domain-specific summaries without forcing every detail into v1.
    pub comparison: Option<Value>,

    /// Forward-compatibility slot for future reproducibility metadata.
    ///
    /// Verification policy and comparison formats will evolve, so unknown fields
    /// are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Privacy signals and redactions applied while building the assertion.
///
/// This record says what the projection policy did and what it assessed. It uses
/// structured statuses instead of booleans so a manifest can distinguish
/// `not-assessed` from `none-detected`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyRecord {
    /// Redactions applied while building the assertion.
    ///
    /// Each redaction identifies a field path and a reason. The list is empty
    /// when no fields were redacted or when the policy chooses not to disclose
    /// redaction details.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redactions: Vec<RedactionRecord>,

    /// Personal-data assessment for disclosed fields.
    ///
    /// A structured status avoids claiming "no personal data" unless a policy or
    /// scanner actually assessed it.
    pub personal_data: DisclosureAssessmentRecord,

    /// Secret assessment for disclosed fields.
    ///
    /// A structured status lets producers distinguish redaction, non-disclosure,
    /// no scan, and no secret detected.
    pub secrets: DisclosureAssessmentRecord,

    /// Forward-compatibility slot for future privacy metadata.
    ///
    /// Privacy policy details are expected to grow as deployment contexts and
    /// regulations vary, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Assessment of whether a class of sensitive data is present.
///
/// This small record is used instead of a boolean because signed privacy claims
/// should be explicit about whether the data was assessed, redacted, detected,
/// not detected, or intentionally undisclosed.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisclosureAssessmentRecord {
    /// Assessment status.
    ///
    /// Suggested values are `not-assessed`, `none-detected`, `detected`,
    /// `redacted`, and `undisclosed`. The field is required so consumers can
    /// avoid interpreting absence as a negative claim.
    pub status: String,

    /// Policy, scanner, or rule set used for the assessment.
    ///
    /// Policy is optional because not all producers have a named scanner or
    /// policy at signing time.
    pub policy: Option<String>,

    /// Assessment timestamp in RFC 3339 format.
    ///
    /// This records when the assessment was made, which can differ from signing
    /// and reproducibility verification times.
    pub assessed_at: Option<String>,

    /// Forward-compatibility slot for future assessment metadata.
    ///
    /// Assessment records may later need confidence, scanner version, or
    /// jurisdiction-specific details, so unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}

/// Redaction applied to a field while projecting the assertion.
///
/// Redaction records are intentionally compact so privacy-relevant decisions can
/// be audited without exposing the redacted value.
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RedactionRecord {
    /// Redacted field path.
    ///
    /// Paths should use assertion JSON field names, for example
    /// `workflow.goalDigest` or `execution.dependencies[0].digest`, so
    /// consumers can locate the omitted context in the signed payload.
    pub field: Option<String>,

    /// Reason the field was redacted.
    ///
    /// Suggested values include `personal-data`, `secret`, `private-path`,
    /// `policy`, and `size`. Custom values should be stable strings rather than
    /// user-facing prose.
    pub reason: Option<String>,

    /// Forward-compatibility slot for future redaction metadata.
    ///
    /// Future redaction records may include policy references or severity, so
    /// unknown fields are preserved.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    #[schemars(skip)]
    pub extra: Map<String, Value>,
}
