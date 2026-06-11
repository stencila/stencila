---
title: Graph Evidence
description: Evidence for a graph edge.
---

This is an evidence type used in Stencila for graph edges.

It records why an edge exists, how confident the assertion is, and optional
provenance details about the observation, assertion, or imported metadata.

The evidence metadata fields have distinct roles:

- `codeLocation` is the exact location in source text where the evidence was
  found, such as a line and column in a code file, manifest, or document.
- `source` is the evidence carrier or authority when it is not sufficiently
  described by `codeLocation.source`, such as an imported graph, attestation,
  external service, execution record, or user assertion.
- `details` is for machine-readable detector-specific context that is not
  represented by core fields, such as parser rules, path expressions,
  dependency groups, package identifiers, or confidence inputs.
- `description` is an optional human-readable fallback for display when a
  useful explanation cannot be derived from the structured fields.


# Properties

The `GraphEvidence` type has these properties:

| Name           | Description                                                                                | Type                                                            | Inherited from          |
| -------------- | ------------------------------------------------------------------------------------------ | --------------------------------------------------------------- | ----------------------- |
| `kind`         | The kind of evidence.                                                                      | [`GraphEvidenceKind`](./graph-evidence-kind.md)                 | -                       |
| `confidence`   | The confidence in the evidence.                                                            | [`GraphEvidenceConfidence`](./graph-evidence-confidence.md)     | -                       |
| `codeLocation` | The exact location in source text where the evidence was found.                            | [`CodeLocation`](./code-location.md)                            | -                       |
| `source`       | The evidence carrier or authority, when not sufficiently represented by the code location. | [`ThingVariant`](./thing-variant.md) \| [`String`](./string.md) | -                       |
| `recordedAt`   | When this evidence was recorded.                                                           | [`Timestamp`](./timestamp.md)                                   | -                       |
| `details`      | Additional machine-readable details about the evidence.                                    | [`Object`](./object.md)                                         | -                       |
| `description`  | Optional human-readable explanation of the evidence.                                       | [`String`](./string.md)                                         | -                       |
| `id`           | The identifier for this item.                                                              | [`String`](./string.md)                                         | [`Entity`](./entity.md) |

# Related

The `GraphEvidence` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `GraphEvidence` type is represented in:

- [JSON-LD](https://stencila.org/GraphEvidence.jsonld)
- [JSON Schema](https://stencila.org/GraphEvidence.schema.json)
- Python class [`GraphEvidence`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`GraphEvidence`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_evidence.rs)
- TypeScript class [`GraphEvidence`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphEvidence.ts)

***

This documentation was generated from [`GraphEvidence.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphEvidence.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
