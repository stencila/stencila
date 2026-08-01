---
title: Graph Evidence Kind
description: The kind of evidence supporting a graph edge.
---

This enumeration classifies how a graph edge was learned or justified.
Variants describe the evidence source or acquisition method, not the graph
relationship itself. This keeps evidence orthogonal to `GraphEdgeKind`, and
allows a single edge to carry several genuinely independent signals.

Use `Observed` narrowly for direct inspection of existing state, such as a
filesystem entry or symlink target. Use `RuntimeAnalysis` for confirmed
relationships observed while executing code. Use `Declared` for explicit authored
fields, including workflow directives, ASTRA contracts, and environment
requirements. Use `StaticAnalysis` for inferred relationships found by
analyzing source or configuration without executing it, such as imports,
conventional lockfile associations, variable flow, or inferred I/O calls.

Parsing, normalization, and resolution enrich the evidence basis that caused
the edge; they do not create additional evidence items. `Resolved` is retained
for relationships where deterministic resolution is itself the sole evidence
basis.


# Members

The `GraphEvidenceKind` type has these members:

| Member            | Description                                                                                                                                         |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Declared`        | Evidence from an explicit authored or schema field declaration, such as a link target, citation target, include source, or media URL.               |
| `Resolved`        | Evidence from deterministic resolution when that resolution is itself the sole evidentiary basis.                                                   |
| `Observed`        | Evidence from direct inspection of existing local state, such as filesystem entries or symlink targets.                                             |
| `Computed`        | Evidence from a deterministic Stencila operation, such as walking document structure, decoding a file, or materializing recorded execution outputs. |
| `Recorded`        | Evidence from persisted Stencila metadata already recorded on a node, such as execution dependencies, execution digests, or execution status.       |
| `StaticAnalysis`  | Evidence inferred by analyzing code or configuration without executing it, rather than copied from an explicit authored relationship field.         |
| `RuntimeAnalysis` | Evidence from a confirmed operation observed while executing code, such as a successful file access, import, or remote response.                    |
| `Imported`        | Evidence imported from an external metadata source, service, graph, or document format.                                                             |
| `UserAssertion`   | Evidence explicitly asserted by a user.                                                                                                             |
| `Attested`        | Evidence from a signed, cryptographic, or otherwise verifiable attestation such as a content credential.                                            |
| `Inferred`        | Evidence from a heuristic or probabilistic inference that has not been directly resolved, observed, or attested.                                    |

# Bindings

The `GraphEvidenceKind` type is represented in:

- [JSON-LD](https://stencila.org/GraphEvidenceKind.jsonld)
- [JSON Schema](https://stencila.org/GraphEvidenceKind.schema.json)
- Python type [`GraphEvidenceKind`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`GraphEvidenceKind`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_evidence_kind.rs)
- TypeScript type [`GraphEvidenceKind`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphEvidenceKind.ts)

***

This documentation was generated from [`GraphEvidenceKind.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphEvidenceKind.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
