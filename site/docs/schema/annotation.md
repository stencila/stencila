---
title: Annotation
description: Annotated content.
---

This is a type used in Stencila Schema for annotated inline content.

It extends [`Mark`](./mark.md) to attach structured authorship and
provenance-aware commentary to a span of content, rather than relying only on
presentational markup. This makes annotations useful in collaborative and
review-oriented document workflows.

Key properties include `annotation`, together with inherited metadata and
provenance features from [`Mark`](./mark.md).


# Analogues

The following external types, elements, or nodes are similar to a `Annotation`:

- HTML [`<mark>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/mark): Approximate HTML analogue for highlighted or annotated spans, though Stencila `Annotation` additionally stores attached block content as the annotation itself.
- JATS [`<annotation>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/annotation.html): Close JATS analogue for attached annotation content.
- Pandoc [`Span`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Span): Closest Pandoc inline container analogue, but annotations there are usually conveyed via attributes rather than a dedicated node with block annotation content.

# Properties

The `Annotation` type has these properties:

| Name         | Description                          | Type                     | Inherited from          |
| ------------ | ------------------------------------ | ------------------------ | ----------------------- |
| `annotation` | The annotation, usually a paragraph. | [`Block`](./block.md)*   | -                       |
| `content`    | The content that is marked.          | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |
| `id`         | The identifier for this item.        | [`String`](./string.md)  | [`Entity`](./entity.md) |

# Related

The `Annotation` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `Annotation` type is represented in:

- [JSON-LD](https://stencila.org/Annotation.jsonld)
- [JSON Schema](https://stencila.org/Annotation.schema.json)
- Python class [`Annotation`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Annotation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/annotation.rs)
- TypeScript class [`Annotation`](https://github.com/stencila/stencila/blob/main/ts/src/types/Annotation.ts)

***

This documentation was generated from [`Annotation.yaml`](https://github.com/stencila/stencila/blob/main/schema/Annotation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
