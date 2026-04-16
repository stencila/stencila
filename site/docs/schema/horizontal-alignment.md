---
title: Horizontal Alignment
description: The horizontal alignment of content.
---

This is an enumeration used in Stencila Schema for horizontal alignment.

It is based on the JATS
[`align`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/attribute/align.html)
attribute and gives Stencila a controlled vocabulary for left, right, center,
justified, and character-based alignment. Using an enumeration allows these
alignment semantics to be mapped consistently across table, layout, and
publishing formats.

See the alignment properties on table- and layout-related types that use this
enumeration.


# Members

The `HorizontalAlignment` type has these members:

| Member           | Description                       |
| ---------------- | --------------------------------- |
| `AlignLeft`      | Left align content.               |
| `AlignRight`     | Right align content.              |
| `AlignJustify`   | Fully justify cell content.       |
| `AlignCenter`    | Center align the cell content.    |
| `AlignCharacter` | Align the content on a character. |

# Bindings

The `HorizontalAlignment` type is represented in:

- [JSON-LD](https://stencila.org/HorizontalAlignment.jsonld)
- [JSON Schema](https://stencila.org/HorizontalAlignment.schema.json)
- Python type [`HorizontalAlignment`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`HorizontalAlignment`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/horizontal_alignment.rs)
- TypeScript type [`HorizontalAlignment`](https://github.com/stencila/stencila/blob/main/ts/src/types/HorizontalAlignment.ts)

***

This documentation was generated from [`HorizontalAlignment.yaml`](https://github.com/stencila/stencila/blob/main/schema/HorizontalAlignment.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
