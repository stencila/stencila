---
title: Block
description: Union type in block content node types.
---

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
- Python type [`Block`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/block.py)
- Rust type [`Block`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/block.rs)
- TypeScript type [`Block`](https://github.com/stencila/stencila/blob/main/ts/src/types/Block.ts)

# Testing

During property-based (a.k.a generative) testing, the variants of the `Block` type are generated using the following strategies[^1] for each complexity level. Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

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

# Source

This documentation was generated from [`Block.yaml`](https://github.com/stencila/stencila/blob/main/schema/Block.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
