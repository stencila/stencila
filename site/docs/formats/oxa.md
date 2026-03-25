---
title: OXA JSON
description: Serialization format for the OXA document model
---

# Introduction

[OXA](https://oxa.dev) is a document model for scientific and technical documents. OXA JSON is the JSON serialization of OXA documents, using a node-tree structure with `type`, `children`, and `value` fields.

Stencila supports OXA JSON as a conversion target to enable interoperability with other tools and platforms that support the OXA initiative. Because both Stencila Schema and OXA aim to represent scientific and technical documents, there is natural overlap. Both schemas model documents as trees of typed nodes representing the structure and content of scientific articles. They share many of the same foundational concepts: paragraphs, headings, emphasis, code blocks, and so on.

Stencila Schema is currently richer than OXA; it defines around 190 node types covering not only static document content but also executable code (`CodeChunk`, `CodeExpression`), control flow (`IfBlock`, `ForBlock`, `CallBlock`), structured data (`Table`, `Figure`, `List`, `Admonition`), editorial workflow (`SuggestionBlock`, `Comment`), and rich inline types (`Link`, `Citation`, `MathInline`). OXA, which is still in early development, defines a smaller set of types focused on core document structure.

For Stencila types that do not yet have a direct OXA equivalent, the codec encodes them to OXA JSON using a structure that mirrors existing OXA conventions (a `type`, `children`, and `data` layout). However, these encodings are somewhat speculative, since the OXA schema is still under active development and has not yet stabilized types for many of these concepts. As OXA matures and adds more types, the number of direct mappings between the two schemas is expected to grow, and conversion fidelity will improve.

# Usage

Use the `.oxa` file extension, or the `--to oxa` or `--from oxa` options, when converting to/from OXA JSON e.g.

```sh
stencila convert doc.myst doc.oxa
stencila convert article.oxa article.docx
```

# Type Mappings

## Equivalent Types

The following Stencila Schema types have direct OXA equivalents:

| Stencila Type                                  | OXA Type                                                                       |
| ---------------------------------------------- | ------------------------------------------------------------------------------ |
| **Root nodes**                                 |                                                                                |
| [`Article`](../schema/article.md)              | [`Document`](https://oxa.dev/articles/documentation/schema/document)           |
| **Block nodes**                                |                                                                                |
| [`Paragraph`](../schema/paragraph.md)          | [`Paragraph`](https://oxa.dev/articles/documentation/schema/paragraph)         |
| [`Heading`](../schema/heading.md)              | [`Heading`](https://oxa.dev/articles/documentation/schema/heading)             |
| [`CodeBlock`](../schema/code-block.md)         | [`Code`](https://oxa.dev/articles/documentation/schema/code)                   |
| [`ThematicBreak`](../schema/thematic-break.md) | [`ThematicBreak`](https://oxa.dev/articles/documentation/schema/thematicbreak) |
| **Inline nodes**                               |                                                                                |
| [`Text`](../schema/text.md)                    | [`Text`](https://oxa.dev/articles/documentation/schema/text)                   |
| [`Emphasis`](../schema/emphasis.md)            | [`Emphasis`](https://oxa.dev/articles/documentation/schema/emphasis)           |
| [`Strong`](../schema/strong.md)                | [`Strong`](https://oxa.dev/articles/documentation/schema/strong)               |
| [`CodeInline`](../schema/code-inline.md)       | [`InlineCode`](https://oxa.dev/articles/documentation/schema/inlinecode)       |
| [`Subscript`](../schema/subscript.md)          | [`Subscript`](https://oxa.dev/articles/documentation/schema/subscript)         |
| [`Superscript`](../schema/superscript.md)      | [`Superscript`](https://oxa.dev/articles/documentation/schema/superscript)     |

## Generic Fallback

Stencila types that do not have a direct OXA equivalent (e.g. `List`, `Table`, `Figure`, `IfBlock`) are encoded using a generic fallback strategy. The generic encoder serializes the node to a JSON object with its Stencila `type` name, any child content recursively encoded into a `children` array, and remaining properties placed into a `data` object.

When decoding OXA JSON, any block-level type not in the direct mapping table is decoded as a `RawBlock` with format `application/vnd.oxa+json` and the original JSON preserved verbatim as content. Unknown inline types are decoded as `Text` nodes with their text content recursively extracted from nested children.

# Implementation

The OXA codec is implemented in the Rust crate [`codec-oxa`](https://github.com/stencila/stencila/blob/main/rust/codec-oxa). It uses [`serde_json`](https://crates.io/crates/serde_json) to parse and generate OXA JSON, with custom encoding and decoding functions for each directly-mapped type and a generic fallback for all other types.

# Limitations

- OXA `classes` fields on nodes are dropped during decoding because Stencila Schema types do not have a general-purpose `classes` property. This is recorded as a `decode:oxa_classes_dropped` loss.
- Stencila types encoded via the generic fallback cannot be reconstructed to their original type on decode; they become `RawBlock` nodes.
- Document metadata mapping is limited to `doi` and `keywords`.
- OXA-specific features such as cross-references and interactive outputs have no Stencila equivalent.
