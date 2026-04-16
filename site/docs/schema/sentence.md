---
title: Sentence
description: A sentence, usually within a `Paragraph`.
---

This is a type used in Stencila Schema for explicit sentence-level content.

It exists so documents can represent sentence boundaries structurally when
that granularity matters for provenance, annotation, review, or
natural-language tooling. This is more precise than inferring sentences from
plain text after the fact.

Key properties include the sentence `content` and any associated authorship or
provenance metadata.


# Analogues

The following external types, elements, or nodes are similar to a `Sentence`:

- [Universal Dependencies sentence span](https://universaldependencies.org/): Close linguistic analogue for explicit sentence segmentation, though UD usually treats sentences as annotation spans rather than document-tree nodes.
- Pandoc [`Span`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Span): Only an approximate Pandoc analogue; Pandoc has no dedicated sentence node and sentence boundaries are usually inferred.

# Properties

The `Sentence` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `content` | The content of the sentence.  | [`Inline`](./inline.md)* | -                       |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |

# Related

The `Sentence` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Sentence` type is represented in:

- [JSON-LD](https://stencila.org/Sentence.jsonld)
- [JSON Schema](https://stencila.org/Sentence.schema.json)
- Python class [`Sentence`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Sentence`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/sentence.rs)
- TypeScript class [`Sentence`](https://github.com/stencila/stencila/blob/main/ts/src/types/Sentence.ts)

***

This documentation was generated from [`Sentence.yaml`](https://github.com/stencila/stencila/blob/main/schema/Sentence.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
