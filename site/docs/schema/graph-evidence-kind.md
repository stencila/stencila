---
title: Graph Evidence Kind
description: The kind of evidence supporting a graph edge.
---

This enumeration classifies how a graph edge was learned or justified.
Variants describe the evidence source or acquisition method, not the graph
relationship itself. This keeps evidence orthogonal to `GraphEdgeKind`, and
allows a single edge to carry several signals, for example an authored
declaration that was also resolved to a concrete local file.

Use `Observed` narrowly for direct inspection of existing state, such as a
filesystem entry or runtime access. Use `StaticAnalysis` for relationships
found by parsing or scanning source/configuration text without executing it.


# Members

The `GraphEvidenceKind` type has these members:

| Member           | Description                                                                                                                                            |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Declared`       | Evidence from an explicit authored or schema field declaration, such as a link target, citation target, include source, or media URL.                  |
| `Resolved`       | Evidence from deterministically resolving a declared locator to a concrete graph node.                                                                 |
| `Observed`       | Evidence from direct inspection of existing local state or runtime behavior, such as filesystem entries, symlink targets, or runtime reads and writes. |
| `Computed`       | Evidence from a deterministic Stencila operation, such as walking document structure, decoding a file, or materializing recorded execution outputs.    |
| `Recorded`       | Evidence from persisted Stencila metadata already recorded on a node, such as execution dependencies, execution digests, or execution status.          |
| `StaticAnalysis` | Evidence from analyzing code, configuration, or environment files without executing them, preferably with a source location when available.            |
| `Imported`       | Evidence imported from an external metadata source, service, graph, or document format.                                                                |
| `UserAssertion`  | Evidence explicitly asserted by a user.                                                                                                                |
| `Attested`       | Evidence from a signed, cryptographic, or otherwise verifiable attestation such as a content credential.                                               |
| `Inferred`       | Evidence from a heuristic or probabilistic inference that has not been directly resolved, observed, or attested.                                       |

# Bindings

The `GraphEvidenceKind` type is represented in:

- [JSON-LD](https://stencila.org/GraphEvidenceKind.jsonld)
- [JSON Schema](https://stencila.org/GraphEvidenceKind.schema.json)
- Python type [`GraphEvidenceKind`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`GraphEvidenceKind`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_evidence_kind.rs)
- TypeScript type [`GraphEvidenceKind`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphEvidenceKind.ts)

***

This documentation was generated from [`GraphEvidenceKind.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphEvidenceKind.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
