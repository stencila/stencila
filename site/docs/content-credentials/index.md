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

Stencila uses standard C2PA assertions where they apply, including actions,
ingredients, and AI disclosure. It also adds a Stencila-specific provenance
assertion for document and execution details that generic C2PA assertions do not
model directly.

## Reading Order

Start with the everyday workflow:

- [Usage](usage) explains how to sign, verify, and inspect assets.
- [Profiles](profiles) explains how to choose how much provenance to disclose.
- [Sidecars](sidecars) explains why some assets need a separate `.c2pa` file.
- [Trust](trust) explains the difference between an intact signature and a
  signer your verifier recognizes.

If you are integrating with Stencila or need exact JSON fields, use the
[Provenance Assertion Reference](provenance-assertion).

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
- **Verifier**: the tool that checks whether the manifest is intact, still
  matches the asset, and is signed by a recognized signer.

If you only need to sign, verify, or choose a privacy profile, the pages above
are enough. The rest of this page is for publishers, reviewers, and tool authors
who need to understand the Stencila-specific payload.

## The Stencila Provenance Assertion

The Stencila provenance assertion is the Stencila-specific part of the
credential. It connects a C2PA manifest back to the Stencila document and
execution model.

It can record the signed asset, root Stencila document node, executed node,
output node, producing activities, attributions, reproducibility context, AI
disclosure, and privacy decisions.

> [!info]
> Most users do not need to read the assertion schema. It is mainly for tool
> authors, publishers, and reviewers who need exact field names or want to
> integrate Stencila credentials into another system.

The shape deliberately follows a compact entity, activity, agent model. The
signed asset, root Stencila node, executed Stencila node, and output Stencila
node are entities, `activities` describe the operations that generated or
exported them, and `attributions` carry role-bearing Stencila authorship. This
keeps the assertion aligned with C2PA's workflow focus, Stencila's `AuthorRole`
model, and general provenance vocabularies such as W3C PROV without forcing
consumers to understand all of Stencila Schema.

The generated schema reference starts at
[Provenance Assertion Reference](provenance-assertion). Use it for the exact
payload fields, record types, required fields, and JSON Schema documentation
links.

## Standard C2PA Assertions

The Stencila assertion is a **Stencila-specific detail and cross-reference
layer**. It should not replace standard C2PA assertions when the same fact can
be represented portably.

Standard assertions let generic C2PA tools understand the broad provenance
story. The `org.stencila.provenance` assertion records Stencila-specific details
such as node IDs, execution digests, provenance counts, workspace run IDs,
definition snapshot hashes, and privacy decisions.

Producers should prefer these standard assertions for ecosystem-visible facts:

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
  `c2pa.ingredient.v3` for `data.csv` with `relationship = "inputTo"` when the
  input is disclosed. Also record Stencila-specific dependency context in
  `execution.dependencies` and `execution.digests`.
- If an article is exported to PDF, describe the public creation or export
  action with `c2pa.actions.v2`. Use `rootNode`, `source`, and `producer` to
  record the root Stencila node type, source revision, codec, and renderer.
- If an LLM contributes text or code, emit `c2pa.ai-disclosure` when model use
  is disclosed. Use `aiDisclosure`, `attributions`, and `provenance` to record
  Stencila author roles and provenance counts.

Some overlap is intentional. It becomes a problem only when a fact is stored
**only** in this custom assertion even though generic C2PA consumers need it. In
that case, emit the standard assertion and use this payload for the
Stencila-specific identifiers, digests, and policy context.

> [!tip]
> A good rule of thumb for producers is: put broadly meaningful facts in
> standard C2PA assertions, and use the Stencila assertion for the Stencila
> identifiers and execution context needed to trace the asset back to a
> document.

## Schema Stability

The `org.stencila.provenance` label is stable for the v1 payload family.

The payload declares a versioned schema URL:

```text
https://stencila.org/stencila-provenance-assertion-v1.schema.json
```

The URL identifies the public v1 contract. Signed manifests can outlive the
Stencila release that produced them, so fields in the published payload should
be treated conservatively.

Compatible v1 changes should be additive: adding optional fields, adding
optional record members, and preserving unknown fields when deserializing and
serializing. Most record types include an `extra` forward-compatibility slot so
newer v1 payloads can pass through older tooling without losing unknown fields.

Breaking changes require a new assertion schema URL, such as a future v2
schema. Examples include removing fields, changing field meanings, changing
required field types, or changing identifier semantics in a way that older
consumers would misinterpret.

## Node Identity

`nodeId` and `persistentId` have distinct meanings:

- `nodeId` is a stabilized structural identifier for the node within the
  rendered document.
- `persistentId` is an author-supplied schema-level identifier when one was set
  on the source node.

Consumers should use `persistentId` for author-managed references and `nodeId`
for structural provenance within a specific rendered document.

## Deferred Fields

The v1 schema includes reproducibility and AI disclosure records, but some
producer behavior is intentionally deferred. Exact reproducibility checks,
workflow attribution, and some AI disclosure assertions depend on additional
runtime context and should not be inferred from field absence.
