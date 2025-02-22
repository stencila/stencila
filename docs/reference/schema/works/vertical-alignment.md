---
title: Vertical Alignment
description: The vertical alignment of content.
config:
  publish:
    ghost:
      type: post
      slug: vertical-alignment
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

Based on the JATS [`valign`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/attribute/valign.html) attribute.

# Members

The `VerticalAlignment` type has these members:

- `AlignBaseline`
- `AlignBottom`
- `AlignTop`
- `AlignMiddle`

# Bindings

The `VerticalAlignment` type is represented in:

- [JSON-LD](https://stencila.org/VerticalAlignment.jsonld)
- [JSON Schema](https://stencila.org/VerticalAlignment.schema.json)
- Python type [`VerticalAlignment`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/vertical_alignment.py)
- Rust type [`VerticalAlignment`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/vertical_alignment.rs)
- TypeScript type [`VerticalAlignment`](https://github.com/stencila/stencila/blob/main/ts/src/types/VerticalAlignment.ts)

# Source

This documentation was generated from [`VerticalAlignment.yaml`](https://github.com/stencila/stencila/blob/main/schema/VerticalAlignment.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
