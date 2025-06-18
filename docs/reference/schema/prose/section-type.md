---
title: Section Type
description: The type of a `Section`.
config:
  publish:
    ghost:
      type: post
      slug: section-type
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

Includes the section types recommended by the JATS XML standard
(https://jats.nlm.nih.gov/archiving/tag-library/1.1d1/n-77u2.html) with additional
values for other section types commonly found in documents.


# Members

The `SectionType` type has these members:

- `Abstract`
- `Summary`
- `Introduction`
- `Materials`
- `Methods`
- `Cases`
- `Results`
- `Discussion`
- `Conclusions`
- `SupplementaryMaterials`
- `Main`
- `Header`
- `Footer`
- `Iteration`

# Bindings

The `SectionType` type is represented in:

- [JSON-LD](https://stencila.org/SectionType.jsonld)
- [JSON Schema](https://stencila.org/SectionType.schema.json)
- Python type [`SectionType`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/section_type.py)
- Rust type [`SectionType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/section_type.rs)
- TypeScript type [`SectionType`](https://github.com/stencila/stencila/blob/main/ts/src/types/SectionType.ts)

# Source

This documentation was generated from [`SectionType.yaml`](https://github.com/stencila/stencila/blob/main/schema/SectionType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
