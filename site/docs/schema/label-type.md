---
title: Label Type
description: An automatic labeling category.
---

This is an enumeration used in Stencila Schema for automatic labeling categories.

It exists so nodes such as figures and tables generated from code can
participate in the correct numbering series without relying on
presentation-specific heuristics.

See properties such as [`CodeChunk.labelType`](./code-chunk.md#labeltype) for the
main use of this enumeration.


# Members

The `LabelType` type has these members:

| Member            | Description |
| ----------------- | ----------- |
| `AppendixLabel`   | -           |
| `FigureLabel`     | -           |
| `SupplementLabel` | -           |
| `TableLabel`      | -           |

# Bindings

The `LabelType` type is represented in:

- [JSON-LD](https://stencila.org/LabelType.jsonld)
- [JSON Schema](https://stencila.org/LabelType.schema.json)
- Python type [`LabelType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`LabelType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/label_type.rs)
- TypeScript type [`LabelType`](https://github.com/stencila/stencila/blob/main/ts/src/types/LabelType.ts)

***

This documentation was generated from [`LabelType.yaml`](https://github.com/stencila/stencila/blob/main/schema/LabelType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
