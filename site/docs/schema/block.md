---
title: Block
description: A union type for block content.
---

This is a union type used in Stencila Schema for block-level document content.

It brings together the node types that can appear where a document model
expects block structure, similar to block-content unions in Markdown, HTML,
JATS, and other document ASTs. The union is central to Stencila's ability to
mix prose, code, figures, tables, instructions, and executable content within
one document model.

Use this type to understand what can appear in `content` arrays on document
and container nodes such as [`Article`](./article.md),
[`Section`](./section.md), and [`Figure`](./figure.md).


# Analogues

The following external types, elements, or nodes are similar to a `Block`:

- [HTML flow content](https://html.spec.whatwg.org/multipage/dom.html#flow-content-2): Broadly analogous to the set of HTML elements allowed in block-flow positions, though Stencila's union is document-AST oriented rather than DOM-category based.
- Pandoc [`Block`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#t:Block): Closest Pandoc union analogue for block-level content.

# Members

The `Block` type has these members:

- [`Admonition`](./admonition.md)
- [`AppendixBreak`](./appendix-break.md)
- [`AudioObject`](./audio-object.md)
- [`CallBlock`](./call-block.md)
- [`Chat`](./chat.md)
- [`ChatMessage`](./chat-message.md)
- [`ChatMessageGroup`](./chat-message-group.md)
- [`Claim`](./claim.md)
- [`CodeBlock`](./code-block.md)
- [`CodeChunk`](./code-chunk.md)
- [`Datatable`](./datatable.md)
- [`Excerpt`](./excerpt.md)
- [`Figure`](./figure.md)
- [`File`](./file.md)
- [`ForBlock`](./for-block.md)
- [`Form`](./form.md)
- [`Heading`](./heading.md)
- [`IfBlock`](./if-block.md)
- [`ImageObject`](./image-object.md)
- [`IncludeBlock`](./include-block.md)
- [`InlinesBlock`](./inlines-block.md)
- [`InstructionBlock`](./instruction-block.md)
- [`Island`](./island.md)
- [`List`](./list.md)
- [`MathBlock`](./math-block.md)
- [`Page`](./page.md)
- [`Paragraph`](./paragraph.md)
- [`PromptBlock`](./prompt-block.md)
- [`QuoteBlock`](./quote-block.md)
- [`RawBlock`](./raw-block.md)
- [`Section`](./section.md)
- [`StyledBlock`](./styled-block.md)
- [`SuggestionBlock`](./suggestion-block.md)
- [`Supplement`](./supplement.md)
- [`Table`](./table.md)
- [`ThematicBreak`](./thematic-break.md)
- [`VideoObject`](./video-object.md)
- [`Walkthrough`](./walkthrough.md)

# Bindings

The `Block` type is represented in:

- [JSON-LD](https://stencila.org/Block.jsonld)
- [JSON Schema](https://stencila.org/Block.schema.json)
- Python type [`Block`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Block`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/block.rs)
- TypeScript type [`Block`](https://github.com/stencila/stencila/blob/main/ts/src/types/Block.ts)

# Testing

During property-based (a.k.a generative) testing, the variants of the `Block` type are generated using the following strategies. Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

::: table

| Variant            | Complexity | Description                                                | Strategy          |
| ------------------ | ---------- | ---------------------------------------------------------- | ----------------- |
| `Admonition`       | Min+       | Do not generate `Admonition` nodes in block content.       | -                 |
|                    | Low+       | Generate `Admonition` nodes in block content.              | Default for level |
| `AppendixBreak`    | Min+       | Do not generate `AppendixBreak` nodes in block content.    | -                 |
| `AudioObject`      | Min+       | Do not generate `AudioObject` nodes in block content.      | -                 |
| `CallBlock`        | Min+       | Do not generate `CallBlock` nodes in block content.        | -                 |
|                    | Low+       | Generate `CallBlock` nodes in block content.               | Default for level |
| `Chat`             | Min+       | Do not generate `Chat` nodes in block content.             | -                 |
| `ChatMessage`      | Min+       | Do not generate `ChatMessage` nodes in block content.      | -                 |
| `ChatMessageGroup` | Min+       | Do not generate `ChatMessageGroup` nodes in block content. | -                 |
| `Claim`            | Min+       | Do not generate `Claim` nodes in block content.            | -                 |
|                    | Low+       | Generate `Claim` nodes in block content.                   | Default for level |
| `CodeChunk`        | Min+       | Do not generate `CodeChunk` nodes in block content.        | -                 |
|                    | Low+       | Generate `CodeChunk` nodes in block content.               | Default for level |
| `Datatable`        | Min+       | Do not generate `Datatable` nodes in block content.        | -                 |
| `Excerpt`          | Min+       | Do not generate `Excerpt` nodes in block content.          | -                 |
| `Figure`           | Min+       | Do not generate `Figure` nodes in block content.           | -                 |
|                    | Low+       | Generate `Figure` nodes in block content.                  | Default for level |
| `File`             | Min+       | Do not generate `File` nodes in block content.             | -                 |
| `ForBlock`         | Min+       | Do not generate `ForBlock` nodes in block content.         | -                 |
|                    | Low+       | Generate `ForBlock` nodes in block content.                | Default for level |
| `Form`             | Min+       | Do not generate `Form` nodes in block content.             | -                 |
| `IfBlock`          | Min+       | Do not generate `IfBlock` nodes in block content.          | -                 |
|                    | Low+       | Generate `IfBlock` nodes in block content.                 | Default for level |
| `ImageObject`      | Min+       | Do not generate `ImageObject` nodes in block content.      | -                 |
| `IncludeBlock`     | Min+       | Do not generate `IncludeBlock` nodes in block content.     | -                 |
|                    | Low+       | Generate `IncludeBlock` nodes in block content.            | Default for level |
| `InlinesBlock`     | Min+       | Do not generate `InlinesBlock` nodes in block content.     | -                 |
| `InstructionBlock` | Min+       | Do not generate `InstructionBlock` nodes in block content. | -                 |
| `Island`           | Min+       | Do not generate `Island` nodes in block content.           | -                 |
| `Page`             | Min+       | Do not generate `Page` nodes in block content.             | -                 |
| `PromptBlock`      | Min+       | Do not generate `PromptBlock` nodes in block content.      | -                 |
| `Section`          | Min+       | Do not generate `Section` nodes in block content.          | -                 |
|                    | Low+       | Generate `Section` nodes in block content.                 | Default for level |
| `SuggestionBlock`  | Min+       | Do not generate `SuggestionBlock` nodes in block content.  | -                 |
| `Supplement`       | Min+       | Do not generate `Supplement` nodes in block content.       | -                 |
| `VideoObject`      | Min+       | Do not generate `VideoObject` nodes in block content.      | -                 |
| `Walkthrough`      | Min+       | Do not generate `Walkthrough` nodes in block content.      | -                 |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on the proptest generation strategies listed.

:::

***

This documentation was generated from [`Block.yaml`](https://github.com/stencila/stencila/blob/main/schema/Block.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
