---
title: "Stencila Provenance Assertion v1"
description: "C2PA provenance assertion payload used by Stencila content credentials."
---

# Stencila Provenance Assertion v1

C2PA provenance assertion payload used by Stencila content credentials.

The assertion records the signed asset, root Stencila document node,
executed Stencila node, output Stencila node, producing activities,
attributions, reproducibility context, AI disclosure, and privacy decisions.

The shape deliberately follows a compact entity, activity, agent model. The
signed asset, root Stencila node, executed Stencila node, and output
Stencila node are entities, `activities` describe the operations that
generated or exported them, and `attributions` carry role-bearing Stencila
authorship. This keeps
the assertion aligned with
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
  the input is disclosed. Also record Stencila-specific dependency context
  in `execution.dependencies` and `execution.digests`.
- If an article is exported to PDF, describe the public creation or export
  action with `c2pa.actions.v2`. Use `rootNode`, `source`, and `producer` to
  record the root Stencila node type, source revision, codec, and renderer.
- If an LLM contributes text or code, emit `c2pa.ai-disclosure` when model
  use is disclosed. Use `aiDisclosure`, `attributions`, and
  `provenance` to record Stencila author roles and provenance counts.

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
| [`producer`](#producer) | [`ProducerRecord`](producer-record) | Yes | Software that produced the C2PA claim and this Stencila assertion. |
| [`rootNode`](#root-node) | [`NodeRecord`](node-record) | Yes | Root Stencila document node containing the signed node. |
| [`executedNode`](#executed-node) | [`NodeRecord`](node-record) | No | Stencila node that was executed to produce `outputNode`. |
| [`outputNode`](#output-node) | [`NodeRecord`](node-record) | No | Stencila output node represented by the signed asset. |
| [`asset`](#asset) | [`AssetRecord`](asset-record) | Yes | The signed C2PA asset entity. |
| [`activities`](#activities) | array<[`ActivityRecord`](activity-record)> | No | Activities that generated, exported, or signed the asset. |
| [`attributions`](#attributions) | array<[`AttributionRecord`](attribution-record)> | No | Role-bearing attribution projected from Stencila authorship. |
| [`source`](#source) | [`SourceRecord`](source-record) | No | Source-control facts for the document or project. |
| [`execution`](#execution) | [`ExecutionRecord`](execution-record) | No | Execution facts for executable outputs. |
| [`workflow`](#workflow) | [`WorkflowRecord`](workflow-record) | No | Workflow, agent, and definition facts supplied by an explicit provenance context. |
| [`environment`](#environment) | [`EnvironmentRecord`](environment-record) | No | Environment facts that help a verifier reproduce the output. |
| [`aiDisclosure`](#ai-disclosure) | [`AiDisclosureRecord`](ai-disclosure-record) | No | Stencila projection of standard C2PA AI disclosure concepts. |
| [`provenance`](#provenance) | [`ProvenanceRecord`](provenance-record) | No | Compact projection of Stencila `ProvenanceCount` values. |
| [`reproducibility`](#reproducibility) | [`ReproducibilityRecord`](reproducibility-record) | Yes | Reproducibility status and comparison details known at signing time. |
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

### `producer`

Software that produced the C2PA claim and this Stencila assertion.

This is distinct from `activities.associatedAttributionIds` and `attributions`.
The producer is the claim generator in the C2PA sense, while attributions
describe who or what is credited with creating, generating, verifying, or
accepting the represented content.

**Type:** [`ProducerRecord`](producer-record) | **Required:** Yes

### `rootNode`

Root Stencila document node containing the signed node.

This gives Stencila-aware consumers a bridge back to the document model
without embedding the full root node JSON. For a root document manifest,
this is also the node exported to the signed asset.

**Type:** [`NodeRecord`](node-record) | **Required:** Yes

### `executedNode`

Stencila node that was executed to produce `outputNode`.

Per-output manifests use this field when the signed asset came from an
executable node, such as a `CodeChunk` that generated an image. It is
omitted for plain document exports and manually signed standalone files.

**Type:** [`NodeRecord`](node-record) | **Required:** No | **Nullable:** Yes

### `outputNode`

Stencila output node represented by the signed asset.

For generated media, this is the node in `executedNode.outputs` whose
bytes were exported and signed, such as an `ImageObject`. Keeping this
separate from `asset` matters because the node is Stencila document
structure, while `asset` is the signed byte rendition.

**Type:** [`NodeRecord`](node-record) | **Required:** No | **Nullable:** Yes

### `asset`

The signed C2PA asset entity.

This record describes the exact bytes that C2PA binds to the manifest
before signing. Keeping it separate from `outputNode` matters because the
same Stencila node may be exported to several assets, and an asset can be
a rendition of a larger document rather than a node itself.

**Type:** [`AssetRecord`](asset-record) | **Required:** Yes

### `activities`

Activities that generated, exported, or signed the asset.

Provenance needs explicit operations, not just a pile of facts. Records
are ordered from earliest to latest, so a rendered executable document can
record execution before export.

When an operation is meaningful outside Stencila, producers should also
record it in
[`c2pa.actions.v2`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_actions).
This field then provides the Stencila activity type, timing, and local
relationships needed for reproducibility.

**Type:** array<[`ActivityRecord`](activity-record)> | **Required:** No

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
in `attributions` and `provenance`.

**Type:** [`AiDisclosureRecord`](ai-disclosure-record) | **Required:** No | **Nullable:** Yes

### `provenance`

Compact projection of Stencila `ProvenanceCount` values.

This is a summary, not a substitute for attributions. It answers "how
much of the represented content was human written, machine written, or
reviewed?" using Stencila's stable provenance categories.

**Type:** [`ProvenanceRecord`](provenance-record) | **Required:** No | **Nullable:** Yes

### `reproducibility`

Reproducibility status and comparison details known at signing time.

Reproducibility is included even when no check was run so
consumers can distinguish "not checked" from "checked and failed" rather
than treating absence as a hidden result.

**Type:** [`ReproducibilityRecord`](reproducibility-record) | **Required:** Yes

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
| [`NodeRecord`](node-record) | Facts about a Stencila node related to the signed asset. |
| [`AssetRecord`](asset-record) | Facts about the signed asset entity. |
| [`ActivityRecord`](activity-record) | Activity that generated, exported, or signed the asset. |
| [`AttributionRecord`](attribution-record) | Role-bearing attribution for an agent. |
| [`AgentRecord`](agent-record) | Agent participating in provenance. |
| [`IdentifierRecord`](identifier-record) | Identifier for an agent or other named entity. |
| [`SourceRecord`](source-record) | Source-control facts for the signed output. |
| [`ExecutionRecord`](execution-record) | Execution facts for executable document nodes. |
| [`ExecutionDigestsRecord`](execution-digests-record) | Digest values corresponding to Stencila `CompilationDigest`. |
| [`KernelRecord`](kernel-record) | Kernel used to execute a Stencila node. |
| [`DependencyRecord`](dependency-record) | Dependency of an execution. |
| [`ExecutionMessageRecord`](execution-message-record) | Message emitted during execution. |
| [`WorkflowRecord`](workflow-record) | Explicit workflow, agent, and definition context. |
| [`DefinitionRecord`](definition-record) | Definition loaded by a workflow. |
| [`EnvironmentRecord`](environment-record) | Environment facts selected for publication under the active privacy profile. |
| [`RuntimeRecord`](runtime-record) | Runtime version relevant to reproduction. |
| [`FileDigestRecord`](file-digest-record) | Digest of a file relevant to reproduction. |
| [`AiDisclosureRecord`](ai-disclosure-record) | Stencila projection of C2PA AI disclosure concepts. |
| [`AiContentProfileRecord`](ai-content-profile-record) | Content profile for AI disclosure. |
| [`ProvenanceRecord`](provenance-record) | Compact projection of Stencila provenance categories. |
| [`ProvenanceCategoryRecord`](provenance-category-record) | Per-category count for a Stencila provenance category. |
| [`ReproducibilityRecord`](reproducibility-record) | Reproducibility status and comparison details known when signing. |
| [`PrivacyRecord`](privacy-record) | Privacy signals and redactions applied while building the assertion. |
| [`RedactionRecord`](redaction-record) | Redaction applied to a field while projecting the assertion. |
| [`DisclosureAssessmentRecord`](disclosure-assessment-record) | Assessment of whether a class of sensitive data is present. |

---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
