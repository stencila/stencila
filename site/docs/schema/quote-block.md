---
title: Quote Block
description: A section quoted from somewhere else.
---

This is a block type used in Stencila Schema for quoted content.

It exists to represent extended quotations as structured block content,
distinct from both inline quotation and untyped paragraphs. This helps
preserve attribution, provenance, and format mappings for quoted material.

Key properties include the quoted `content` and any citation or source
metadata associated with the quotation.


# Analogues

The following external types, elements, or nodes are similar to a `QuoteBlock`:

- HTML [`<blockquote>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote)
- JATS [`<disp-quote>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/disp-quote.html)
- Pandoc [`BlockQuote`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:BlockQuote)
- MDAST [`Blockquote`](https://github.com/syntax-tree/mdast#blockquote): Closest MDAST analogue for block quotations, though Stencila additionally supports structured source metadata.

# Properties

The `QuoteBlock` type has these properties:

| Name         | Description                                                    | Type                                               | Inherited from          |
| ------------ | -------------------------------------------------------------- | -------------------------------------------------- | ----------------------- |
| `source`     | The source of the quote.                                       | [`Citation`](./citation.md) \| [`Text`](./text.md) | -                       |
| `content`    | The content of the quote.                                      | [`Block`](./block.md)*                             | -                       |
| `authors`    | The authors of the quote.                                      | [`Author`](./author.md)*                           | -                       |
| `provenance` | A summary of the provenance of the content within the section. | [`ProvenanceCount`](./provenance-count.md)*        | -                       |
| `id`         | The identifier for this item.                                  | [`String`](./string.md)                            | [`Entity`](./entity.md) |

# Related

The `QuoteBlock` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `QuoteBlock` type is represented in:

- [JSON-LD](https://stencila.org/QuoteBlock.jsonld)
- [JSON Schema](https://stencila.org/QuoteBlock.schema.json)
- Python class [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/quote_block.rs)
- TypeScript class [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/QuoteBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `QuoteBlock` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                                 | Strategy                      |
| --------- | ---------- | ----------------------------------------------------------- | ----------------------------- |
| `content` | Min+       | Generate a single arbitrary paragraph.                      | `vec_paragraphs(1)`           |
|           | Low+       | Generate up to two arbitrary, non-recursive, block nodes.   | `vec_blocks_non_recursive(2)` |
|           | High+      | Generate up to four arbitrary, non-recursive, block nodes.  | `vec_blocks_non_recursive(4)` |
|           | Max        | Generate up to eight arbitrary, non-recursive, block nodes. | `vec_blocks_non_recursive(8)` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`QuoteBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/QuoteBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
