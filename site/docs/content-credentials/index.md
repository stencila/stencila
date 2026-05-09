---
title: "Stencila Provenance Assertion v1"
description: "C2PA provenance assertion payload used by Stencila content credentials."
---

# Stencila Provenance Assertion v1

C2PA provenance assertion payload used by Stencila content credentials.

The assertion records the signed asset, represented Stencila document,
producing activity, attributions, reproducibility context, AI disclosure, and
privacy decisions.

The shape deliberately follows a compact entity, activity, agent model. The
signed asset and represented Stencila document are entities, `activity`
describes the operation that generated or exported them, and `attributions`
carry role-bearing Stencila authorship. This keeps the assertion aligned with
C2PA's workflow focus, Stencila's `AuthorRole` model, and general provenance
vocabularies such as W3C PROV without forcing consumers to understand all of
Stencila Schema.

## Relationship to standard C2PA assertions

This Stencila assertion is a **Stencila-specific detail and cross-reference
layer**. It should not replace standard C2PA assertions when the same fact
can be represented portably. Standard assertions let generic C2PA tools
understand the broad provenance story, while `org.stencila.provenance`
records Stencila-specific details such as node IDs, execution digests,
provenance counts, workspace run IDs, definition snapshot hashes, and
privacy decisions.

Producers should prefer these standard assertions for ecosystem-visible
facts:

- [`c2pa.actions.v2`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_actions)
  for public action history such as creation, opening, placement, export, or
  transformation.
- [`c2pa.ingredient.v3`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_ingredient)
  for assets or data used as inputs, components, parents, or process inputs.
- [`c2pa.ai-disclosure`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_ai_disclosure)
  when AI model use is disclosed.

Use this assertion to connect those portable assertions back to Stencila's
document and execution model. For example:

- If a code chunk reads `data.csv` and produces `figure.png`, emit
  `c2pa.ingredient.v3` for `data.csv` with `relationship = "inputTo"` when
  the input is disclosed. Also record the input's Stencila node ID, digest,
  redaction status, and dependency context in `inputs` and `execution`.
- If an article is exported to PDF, describe the public creation or export
  action with `c2pa.actions.v2`. Use `document`, `source`, and `producer` to
  record the Stencila node type, source revision, codec, and renderer.
- If an LLM contributes text or code, emit `c2pa.ai-disclosure` when model
  use is disclosed. Use `aiDisclosure`, `attributions`, and
  `provenanceSummary` to record Stencila author roles and provenance counts.

Some overlap is intentional. It becomes a problem only when a fact is stored
**only** in this custom assertion even though generic C2PA consumers need it.
In that case, emit the standard assertion and use this payload for the
Stencila-specific identifiers, digests, and policy context.

This reference is generated from the Rust wire schema used for the `org.stencila.provenance` C2PA assertion. It documents the public JSON payload shape, including why each field exists and how records relate to Stencila authorship and provenance concepts.

## Payload Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`schema`](#schema) | `string` | Yes | Schema URL identifying the payload family. |
| [`version`](#version) | `integer` | Yes | Numeric payload compatibility version. |
| [`profile`](#profile) | `string` | Yes | High-level assertion profile. |
| [`producer`](#producer) | [`ProducerRecord`](producer-record) | Yes | Software that produced the C2PA claim and this Stencila assertion. |
| [`asset`](#asset) | [`AssetRecord`](asset-record) | Yes | The signed C2PA asset entity. |
| [`document`](#document) | [`DocumentRecord`](document-record) | Yes | The Stencila document node or work represented by the signed asset. |
| [`activity`](#activity) | [`ActivityRecord`](activity-record) | Yes | Activity that generated, exported, or signed the asset. |
| [`attributions`](#attributions) | array<[`AttributionRecord`](attribution-record)> | No | Role-bearing attribution projected from Stencila authorship. |
| [`source`](#source) | [`SourceRecord`](source-record) | No | Source-control facts for the document or project. |
| [`execution`](#execution) | [`ExecutionRecord`](execution-record) | No | Execution facts for executable outputs. |
| [`workflow`](#workflow) | [`WorkflowRecord`](workflow-record) | No | Workflow, agent, and definition facts supplied by an explicit provenance context. |
| [`environment`](#environment) | [`EnvironmentRecord`](environment-record) | No | Environment facts that help a verifier reproduce the output. |
| [`inputs`](#inputs) | array<[`IoRecord`](io-record)> | No | Inputs used by the activity. |
| [`outputs`](#outputs) | array<[`IoRecord`](io-record)> | No | Outputs produced by the same activity. |
| [`aiDisclosure`](#ai-disclosure) | [`AiDisclosureRecord`](ai-disclosure-record) | No | Stencila projection of standard C2PA AI disclosure concepts. |
| [`provenanceSummary`](#provenance-summary) | [`ProvenanceSummaryRecord`](provenance-summary-record) | No | Compact projection of Stencila `ProvenanceCount` values. |
| [`verification`](#verification) | [`VerificationRecord`](verification-record) | Yes | Reproducibility status and comparison details known at signing time. |
| [`privacy`](#privacy) | [`PrivacyRecord`](privacy-record) | Yes | Privacy policy results and redactions applied while building the assertion. |

### `schema`

Schema URL identifying the payload family.

The URL gives validators a dereferenceable schema artifact and a stable
human-facing identifier. It is retained even though `version` is also
present because C2PA assertions are often inspected outside Stencila and
a URL is easier for generic tooling to display or archive.

**Type:** `string` | **Required:** Yes

### `version`

Numeric payload compatibility version.

This is fixed at `1` for the first public contract. It exists so Stencila
can later mint refined v1 schema URLs for documentation or optional
additions without causing every exact-URL verifier to report an otherwise
compatible payload as unknown.

**Type:** `integer` | **Required:** Yes

### `profile`

High-level assertion profile.

The profile lets a verifier choose the relevant interpretation without
having to infer it from the media type alone. The initial vocabulary is
`asset`, `document-export`, and `computational-output`; reverse-DNS
extension values are allowed for workflows that need a narrower profile.

**Type:** `string` | **Required:** Yes

### `producer`

Software that produced the C2PA claim and this Stencila assertion.

This is distinct from `activity.associatedAttributionIds` and `attributions`.
The producer is the claim generator in the C2PA sense, while attributions
describe who or what is credited with creating, generating, verifying, or
accepting the represented content.

**Type:** [`ProducerRecord`](producer-record) | **Required:** Yes

### `asset`

The signed C2PA asset entity.

This record describes the exact bytes that C2PA binds to the manifest
before signing. Keeping it separate from `document` matters because the
same Stencila node may be exported to several assets, and an asset can be
a rendition of a larger document rather than the document itself.

**Type:** [`AssetRecord`](asset-record) | **Required:** Yes

### `document`

The Stencila document node or work represented by the signed asset.

This gives Stencila-aware consumers a bridge back to the document model
without embedding the full node JSON. The fields are intentionally stable
identifiers, labels, and digests rather than mutable presentation details.

**Type:** [`DocumentRecord`](document-record) | **Required:** Yes

### `activity`

Activity that generated, exported, or signed the asset.

Provenance needs an explicit operation, not just a pile of facts. This
record is the point where inputs, outputs, agents, timing, execution, and
workflow details can be related without flattening them into ambiguous
sibling objects.

When the operation is meaningful outside Stencila, producers should also
record it in
[`c2pa.actions.v2`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_actions).
This field then provides the Stencila activity kind, timing, and local
relationships needed for reproducibility.

**Type:** [`ActivityRecord`](activity-record) | **Required:** Yes

### `attributions`

Role-bearing attribution projected from Stencila authorship.

Stencila documents use `Author` and `AuthorRole` to record human,
organizational, and software contributions. This vector preserves that
concept in a compact form so v1 can answer both "who authored this?" and
"what role did this person or system play?" without later adding a
parallel authorship model.

**Type:** array<[`AttributionRecord`](attribution-record)> | **Required:** No

### `source`

Source-control facts for the document or project.

Source state is useful for reproducibility and review, but is kept in an
optional record because many exports are not produced from a public or
even local version-control checkout.

**Type:** [`SourceRecord`](source-record) | **Required:** No | **Nullable:** Yes

### `execution`

Execution facts for executable outputs.

This captures Stencila-specific execution state for code-backed content.
It remains separate from `activity` because not every provenance activity
is a code execution, and one workflow activity may include execution plus
rendering, verification, or signing steps.

**Type:** [`ExecutionRecord`](execution-record) | **Required:** No | **Nullable:** Yes

### `workflow`

Workflow, agent, and definition facts supplied by an explicit provenance context.

Workflow details are optional and may be privacy-sensitive. When present,
they explain the higher-level orchestration around the activity, such as
which workspace workflow run, definition snapshot, or artifact contributed.

**Type:** [`WorkflowRecord`](workflow-record) | **Required:** No | **Nullable:** Yes

### `environment`

Environment facts that help a verifier reproduce the output.

These are selected under a privacy policy because host and runtime
details can reveal private infrastructure. The record focuses on durable,
reproducibility-relevant facts such as container images, runtimes, and
lockfile digests.

**Type:** [`EnvironmentRecord`](environment-record) | **Required:** No | **Nullable:** Yes

### `inputs`

Inputs used by the activity.

Inputs are entities in the provenance sense. They can represent files,
datasets, parameters, prompts, or prior document nodes. Optional IDs allow
`activity.usedInputIds` to point at them without requiring every consumer
to parse domain-specific names.

Disclosed asset or data inputs should usually also be represented as
[`c2pa.ingredient.v3`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_ingredient)
assertions. Use `relationship = "inputTo"` for data used by a
computational process, `relationship = "componentOf"` for composed
content, and `relationship = "parentOf"` for a source asset that was
opened or derived from. Keep Stencila-specific details such as node IDs,
dependency relation, prompt digests, access policy, and redaction status
here.

**Type:** array<[`IoRecord`](io-record)> | **Required:** No

### `outputs`

Outputs produced by the same activity.

Outputs let the assertion describe siblings of the signed asset, for
example a table and a figure generated by the same code chunk. The signed
asset should also be represented by `asset`; outputs are for the broader
operation context.

Outputs that become separately signed assets can later appear as
`c2pa.ingredient.v3` entries in downstream manifests. This record is for
linking those outputs to the Stencila activity and document nodes that
produced them.

**Type:** array<[`IoRecord`](io-record)> | **Required:** No

### `aiDisclosure`

Stencila projection of standard C2PA AI disclosure concepts.

This field is not a replacement for the standard `c2pa.ai-disclosure`
assertion. It exists so Stencila-specific provenance can cross-reference
model and human-oversight details in the same payload while producers are
still encouraged to emit the standard assertion when model use is
disclosed.

When both are present, `standardAssertion` can identify the corresponding
[`c2pa.ai-disclosure`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_ai_disclosure)
assertion, while Stencila-specific role and content attribution remains
in `attributions` and `provenanceSummary`.

**Type:** [`AiDisclosureRecord`](ai-disclosure-record) | **Required:** No | **Nullable:** Yes

### `provenanceSummary`

Compact projection of Stencila `ProvenanceCount` values.

This is a summary, not a substitute for attributions. It answers "how
much of the represented content was human written, machine written, or
reviewed?" using Stencila's stable provenance categories.

**Type:** [`ProvenanceSummaryRecord`](provenance-summary-record) | **Required:** No | **Nullable:** Yes

### `verification`

Reproducibility status and comparison details known at signing time.

Verification is included even when no reproducibility check was run so
consumers can distinguish "not checked" from "checked and failed" rather
than treating absence as a hidden result.

**Type:** [`VerificationRecord`](verification-record) | **Required:** Yes

### `privacy`

Privacy policy results and redactions applied while building the assertion.

C2PA provenance is opt-in and can carry sensitive data. This record makes
the projection policy explicit and avoids over-claiming with bare booleans
such as "contains no secrets" unless an assessment actually ran.

**Type:** [`PrivacyRecord`](privacy-record) | **Required:** Yes


## Record Types

| Record | Description |
|--------|-------------|
| [`ProducerRecord`](producer-record) | Producer metadata embedded in the assertion. |
| [`AssetRecord`](asset-record) | Facts about the signed asset entity. |
| [`DocumentRecord`](document-record) | Facts about the Stencila node or work represented by the signed asset. |
| [`ExecutionDigestRecord`](execution-digest-record) | Digest values corresponding to Stencila `CompilationDigest`. |
| [`ActivityRecord`](activity-record) | Activity that generated, exported, or signed the asset. |
| [`AttributionRecord`](attribution-record) | Role-bearing attribution for an agent. |
| [`AgentRecord`](agent-record) | Agent participating in provenance. |
| [`IdentifierRecord`](identifier-record) | Identifier for an agent or other named entity. |
| [`SourceRecord`](source-record) | Source-control facts for the signed output. |
| [`ExecutionRecord`](execution-record) | Execution facts for executable document nodes. |
| [`KernelRecord`](kernel-record) | Kernel used to execute a Stencila node. |
| [`DependencyRecord`](dependency-record) | Dependency of an execution. |
| [`ExecutionMessageRecord`](execution-message-record) | Message emitted during execution. |
| [`WorkflowRecord`](workflow-record) | Explicit workflow, agent, and definition context. |
| [`DefinitionRecord`](definition-record) | Definition loaded by a workflow. |
| [`EnvironmentRecord`](environment-record) | Environment facts selected for publication under the active privacy profile. |
| [`RuntimeRecord`](runtime-record) | Runtime version relevant to reproduction. |
| [`FileDigestRecord`](file-digest-record) | Digest of a file relevant to reproduction. |
| [`IoRecord`](io-record) | Input or output entity used by a provenance activity. |
| [`AiDisclosureRecord`](ai-disclosure-record) | Stencila projection of C2PA AI disclosure concepts. |
| [`AiContentProfileRecord`](ai-content-profile-record) | Content profile for AI disclosure. |
| [`ProvenanceSummaryRecord`](provenance-summary-record) | Compact projection of Stencila provenance categories. |
| [`ProvenanceCategoryRecord`](provenance-category-record) | Per-category count for a Stencila provenance category. |
| [`VerificationRecord`](verification-record) | Reproducibility status and comparison details known when signing. |
| [`PrivacyRecord`](privacy-record) | Privacy signals and redactions applied while building the assertion. |
| [`RedactionRecord`](redaction-record) | Redaction applied to a field while projecting the assertion. |
| [`DisclosureAssessmentRecord`](disclosure-assessment-record) | Assessment of whether a class of sensitive data is present. |

---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
