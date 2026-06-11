---
title: Graph Evidence Confidence
description: The confidence level for graph evidence.
---

# Members

The `GraphEvidenceConfidence` type has these members:

| Member    | Description                                                                                                 |
| --------- | ----------------------------------------------------------------------------------------------------------- |
| `Low`     | Plausible but weakly supported or not independently verified.                                               |
| `Medium`  | Supported by one reliable signal, but not enough to treat as definitive.                                    |
| `High`    | Supported by multiple corroborating signals or a deterministic local check.                                 |
| `Certain` | Directly observed, deterministically proven, or cryptographically attested with no known contrary evidence. |

# Bindings

The `GraphEvidenceConfidence` type is represented in:

- [JSON-LD](https://stencila.org/GraphEvidenceConfidence.jsonld)
- [JSON Schema](https://stencila.org/GraphEvidenceConfidence.schema.json)
- Python type [`GraphEvidenceConfidence`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`GraphEvidenceConfidence`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_evidence_confidence.rs)
- TypeScript type [`GraphEvidenceConfidence`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphEvidenceConfidence.ts)

***

This documentation was generated from [`GraphEvidenceConfidence.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphEvidenceConfidence.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
