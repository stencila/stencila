---
title: Vertical Alignment
description: The vertical alignment of content.
---

This is an enumeration used in Stencila Schema for vertical alignment.

It is based on the JATS
[`valign`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/attribute/valign.html)
attribute and gives Stencila a controlled vocabulary for baseline, top,
middle, and bottom alignment. Using an enumeration allows these alignment
semantics to be mapped consistently across table, layout, and publishing
formats.

See the alignment properties on table- and layout-related types that use this
enumeration.


# Members

The `VerticalAlignment` type has these members:

| Member          | Description            |
| --------------- | ---------------------- |
| `AlignBaseline` | Aligned to a baseline. |
| `AlignBottom`   | Aligned with bottom.   |
| `AlignTop`      | Aligned with top.      |
| `AlignMiddle`   | Centered vertically.   |

# Bindings

The `VerticalAlignment` type is represented in:

- [JSON-LD](https://stencila.org/VerticalAlignment.jsonld)
- [JSON Schema](https://stencila.org/VerticalAlignment.schema.json)
- Python type [`VerticalAlignment`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`VerticalAlignment`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/vertical_alignment.rs)
- TypeScript type [`VerticalAlignment`](https://github.com/stencila/stencila/blob/main/ts/src/types/VerticalAlignment.ts)

***

This documentation was generated from [`VerticalAlignment.yaml`](https://github.com/stencila/stencila/blob/main/schema/VerticalAlignment.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
