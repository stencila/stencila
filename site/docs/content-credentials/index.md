---
title: Stencila Content Credentials
description: Signed provenance for Stencila documents, figures, and other research assets.
---

# Content Credentials

Research outputs often move through many hands, tools, and file formats. A
figure may start as a code chunk in a Stencila document, read data from a local
file or repository, be rendered to PNG or PDF, be revised for submission, and
then be downloaded by a reader months later.

By that point, ordinary file metadata is rarely enough. Reviewers, editors,
collaborators, and future readers may need to ask:

- Where did this asset come from?
- Which tool produced or changed it?
- Was AI involved?
- Which document node, source revision, code execution, or dataset does it
  represent?
- Has the signed asset or its provenance data changed since signing?

Content Credentials give those questions a portable place to live. In Stencila,
a Content Credential is a signed C2PA manifest that records selected provenance
for a document, figure, or other exported asset. The manifest can travel inside
the file when the format supports embedded credentials, or beside the file as a
`.c2pa` sidecar.

The [Coalition for Content Provenance and Authenticity](https://c2pa.org/)
(C2PA) defines the open standard Stencila uses. The standard is designed for
interoperability: a credential produced by Stencila should be inspectable by
other C2PA-aware tools, and Stencila can verify credentials created elsewhere.

The [Content Authenticity Initiative](https://contentauthenticity.org/) (CAI)
describes Content Credentials as a kind of "nutrition label" for digital
content. That analogy is useful for research: the credential gives readers more
context for judgment, but it does not make the judgment for them.

> [!warning]
> A Content Credential can show that provenance is intact, still matches the
> signed file, and was signed by a recognized signer under a verifier's policy.
> It does not prove that a scientific claim is correct, validate the analysis,
> or replace peer review.

## What Stencila Records

Stencila adds Content Credentials to exported assets so that provenance from an
executable document can be carried into downstream review and publishing
workflows.

For example, a signed figure can record that it was exported by Stencila from a
specific document node, produced by a particular execution, and associated with:

- selected inputs and source references
- the software and rendering workflow that produced it
- code execution details and reproducibility status known at signing time
- AI-use disclosures and human or software attributions
- privacy decisions and redactions made before signing

The amount of detail depends on the selected [profile](profiles).
The signing identity depends on the selected [signing backend](signing), such
as a local self-signed identity or Stencila Cloud signing.

Stencila records its detailed provenance as a Stencila Schema `Graph` in the
`org.stencila.provenance` assertion. The same graph is also projected into
standard C2PA assertions where they apply, including actions, ingredients, and
AI disclosure, so generic C2PA tools can still understand the broad provenance
story. Stencila also records `claim_generator_info` so verifiers can distinguish
locally generated and signed manifests from manifests generated or signed by
Stencila Cloud.

## Reading Order

Start with the everyday workflow:

- [Usage](usage) explains how to sign, verify, and inspect assets.
- [Profiles](profiles) explains how to choose how much provenance to disclose.
- [Signing](signing) explains local signing, Cloud signing, and how manifests
  identify where provenance was generated.
- [Sidecars](sidecars) explains why some assets need a separate `.c2pa` file.
- [Trust](trust) explains the difference between an intact signature and a
  signer your verifier recognizes.

If you are integrating with Stencila or need exact JSON fields, inspect the
`org.stencila.provenance` assertion as a Stencila Schema `Graph`. The payload
includes standalone JSON metadata:

```text
$schema: https://stencila.org/v<stencila-version>/Graph.schema.json
@context: https://stencila.org/v<stencila-version>/context.jsonld
```

The version segment follows the Stencila release that produced the credential.

> [!tip]
> For most research users, the main decision is not "how strong should the
> signature be?" but "which audience should see this provenance?" Use
> [Profiles](profiles) to make that privacy choice before sharing signed files.

## Terms You Will See

These words appear throughout the Content Credentials pages:

- **Asset**: the file being signed, such as a figure, PDF, image, or exported
  document.
- **Manifest**: the structured C2PA data attached to the asset or stored in a
  sidecar.
- **Assertion**: one part of a manifest, such as a record of actions, inputs,
  AI disclosure, or Stencila provenance.
- **Signer**: the person, organization, device, or software identity that signs
  the manifest.
- **Claim generator**: the software or service that generated the C2PA claim.
  Stencila records extra fields to distinguish client-generated, Cloud-signed
  manifests from Cloud-generated, Cloud-signed manifests.
- **Verifier**: the tool that checks whether the manifest is intact, still
  matches the asset, and is signed by a recognized signer.

If you only need to sign, verify, or choose a privacy profile, the pages above
are enough. The rest of this page is for publishers, reviewers, and tool authors
who need to understand the Stencila-specific payload.

## The Stencila Provenance Graph

The Stencila provenance graph is the Stencila-specific part of the credential.
It connects a C2PA manifest back to the Stencila document, execution, source,
and asset model.

It can record the signed asset, Stencila document nodes, source files,
executions, output assets, ingredients, producer software, AI models,
attributions, reproducibility context, C2PA-derived provenance from input
assets, and privacy decisions.

> [!info]
> Most users do not need to read the graph schema. It is mainly for tool
> authors, publishers, and reviewers who need to integrate Stencila credentials
> into another system.

The payload uses the existing Stencila Schema `Graph` type rather than a
separate provenance-specific schema. Nodes represent assets, document nodes,
people, organizations, software, models, and other provenance resources. Edges
describe relationships such as generated, derived-into, used-by, cited-by,
imported-by, read-by, written-to, part-of, or attributed-to. Edge evidence
records the specific activity, execution, source, workflow, environment, C2PA,
AI-use, reproducibility, and privacy facts that support those relationships.

This keeps Stencila's detailed provenance in the same schema family as the rest
of the document model while leaving portable C2PA facts available through
standard assertions.

## Standard C2PA Projections

The graph is Stencila's detailed provenance payload, but it should not strand
portable facts inside a Stencila-only assertion. Stencila projects the graph
into standard C2PA assertions for ecosystem-visible facts:

- `claim_generator_info` for the software or service that generated the C2PA
  claim. Stencila writes `org.stencila.generated_by` and
  `org.stencila.signed_by` extension keys here so generic C2PA tooling can
  still display the claim generator while Stencila-aware tooling can
  distinguish local and Cloud workflows.
- [`c2pa.actions.v2`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_actions)
  for public action history such as creation, opening, placement, export, or
  transformation.
- [`c2pa.ingredient.v3`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_ingredient)
  for assets or data used as inputs, components, parents, or process inputs.
- [`c2pa.ai-disclosure`](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_ai_disclosure)
  when AI model use is disclosed.

For example:

- If a code chunk reads `data.csv` and produces `figure.png`, the graph can
  record both the Stencila execution context and a disclosed ingredient edge.
  The C2PA projection can then emit `c2pa.ingredient.v3` for `data.csv`.
- If an article is exported to PDF, graph edges can record the source document,
  producer software, export activity, and generated asset. The C2PA projection
  can then emit public creation or export actions.
- If an LLM contributes text or code, the graph can record model and attribution
  context. The C2PA projection can then emit `c2pa.ai-disclosure` when the
  model use is disclosed.

Some overlap is intentional. It becomes a problem only when a fact is stored
**only** in the graph even though generic C2PA consumers need it. In that case,
the graph should remain the detailed source of truth and the producer should
also emit the corresponding standard C2PA assertion.

> [!tip]
> A good rule of thumb for producers is: build the graph first, then project
> broadly meaningful facts into standard C2PA assertions.

## Schema Stability

The `org.stencila.provenance` assertion label identifies a Stencila Schema
`Graph` payload.

The payload declares versioned JSON Schema and JSON-LD context URLs:

```text
$schema: https://stencila.org/v<stencila-version>/Graph.schema.json
@context: https://stencila.org/v<stencila-version>/context.jsonld
```

The version segment follows the Stencila release that produced the credential.

Signed manifests can outlive the Stencila release that produced them, so graph
consumers should treat fields conservatively and preserve unknown fields where
possible. Compatibility follows Stencila Schema compatibility: additive graph
fields, new optional node details, and new optional evidence details should not
break older consumers that ignore what they do not understand.

## Graph Node Identity

Graph node identifiers and Stencila document identifiers have distinct
meanings:

- `GraphNode.id` is a graph-local identifier used by edges in the signed
  provenance graph.
- Stencila document node identifiers and persistent identifiers, when present
  in node details, identify the underlying document structure or author-managed
  reference.

Consumers should use graph node identifiers for graph traversal and document
node identifiers for references back into the source Stencila document.

## Deferred Fields

The graph can include reproducibility and AI disclosure evidence, but some
producer behavior is intentionally deferred. Exact reproducibility checks,
workflow attribution, and some AI disclosure assertions depend on additional
runtime context and should not be inferred from field absence.
