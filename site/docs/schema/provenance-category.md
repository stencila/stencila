---
title: Provenance Category
description: A category of content provenance.
---

This is an enumeration used in Stencila Schema for summarizing the provenance of
content.

It exists to classify content according to human and machine involvement using
a stable vocabulary that can be counted, rendered, and analyzed consistently
across documents.

See [`ProvenanceCount.category`](./provenance-count.md#category) and related
provenance summaries for where this enumeration is used.


# Members

The `ProvenanceCategory` type has these members:

| Member   | Description                                      |
| -------- | ------------------------------------------------ |
| `HwHeHv` | Human written, edited and verified.              |
| `HwHe`   | Human written and edited.                        |
| `HwHv`   | Human written and verified.                      |
| `Hw`     | Human written.                                   |
| `HwMv`   | Human written, machine verified.                 |
| `MwHeHv` | Machine written, human edited, human verified.   |
| `MwHe`   | Machine written, human edited.                   |
| `MwHeMv` | Machine written, human edited, machine verified. |
| `HwMeHv` | Human written, machine edited, human verified.   |
| `HwMe`   | Human written, machine edited.                   |
| `HwMeMv` | Human written, machine edited, machine verified. |
| `MwHv`   | Machine written, human verified.                 |
| `MwMeHv` | Machine written, machine edited, human verified. |
| `Mw`     | Machine written.                                 |
| `MwMv`   | Machine written and verified.                    |
| `MwMe`   | Machine written and edited.                      |
| `MwMeMv` | Machine written, edited and verified.            |

# Bindings

The `ProvenanceCategory` type is represented in:

- [JSON-LD](https://stencila.org/ProvenanceCategory.jsonld)
- [JSON Schema](https://stencila.org/ProvenanceCategory.schema.json)
- Python type [`ProvenanceCategory`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ProvenanceCategory`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/provenance_category.rs)
- TypeScript type [`ProvenanceCategory`](https://github.com/stencila/stencila/blob/main/ts/src/types/ProvenanceCategory.ts)

***

This documentation was generated from [`ProvenanceCategory.yaml`](https://github.com/stencila/stencila/blob/main/schema/ProvenanceCategory.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
