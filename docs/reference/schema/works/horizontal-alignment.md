---
title: Horizontal Alignment
description: The horizontal alignment of content.
config:
  publish:
    ghost:
      type: post
      slug: horizontal-alignment
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

Based on the JATS [`align`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/attribute/align.html) attribute.

# Members

The `HorizontalAlignment` type has these members:

- `AlignLeft`
- `AlignRight`
- `AlignJustify`
- `AlignCenter`
- `AlignCharacter`

# Bindings

The `HorizontalAlignment` type is represented in:

- [JSON-LD](https://stencila.org/HorizontalAlignment.jsonld)
- [JSON Schema](https://stencila.org/HorizontalAlignment.schema.json)
- Python type [`HorizontalAlignment`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/horizontal_alignment.py)
- Rust type [`HorizontalAlignment`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/horizontal_alignment.rs)
- TypeScript type [`HorizontalAlignment`](https://github.com/stencila/stencila/blob/main/ts/src/types/HorizontalAlignment.ts)

# Source

This documentation was generated from [`HorizontalAlignment.yaml`](https://github.com/stencila/stencila/blob/main/schema/HorizontalAlignment.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
