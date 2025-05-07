---
title: Block
description: Union type in block content node types.
config:
  publish:
    ghost:
      type: post
      slug: block
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

# Members

The `Block` type has these members:

- [`Admonition`](https://stencila.ghost.io/docs/reference/schema/admonition)
- [`AudioObject`](https://stencila.ghost.io/docs/reference/schema/audio-object)
- [`CallBlock`](https://stencila.ghost.io/docs/reference/schema/call-block)
- [`Chat`](https://stencila.ghost.io/docs/reference/schema/chat)
- [`ChatMessage`](https://stencila.ghost.io/docs/reference/schema/chat-message)
- [`ChatMessageGroup`](https://stencila.ghost.io/docs/reference/schema/chat-message-group)
- [`Claim`](https://stencila.ghost.io/docs/reference/schema/claim)
- [`CodeBlock`](https://stencila.ghost.io/docs/reference/schema/code-block)
- [`CodeChunk`](https://stencila.ghost.io/docs/reference/schema/code-chunk)
- [`Excerpt`](https://stencila.ghost.io/docs/reference/schema/excerpt)
- [`Figure`](https://stencila.ghost.io/docs/reference/schema/figure)
- [`File`](https://stencila.ghost.io/docs/reference/schema/file)
- [`ForBlock`](https://stencila.ghost.io/docs/reference/schema/for-block)
- [`Form`](https://stencila.ghost.io/docs/reference/schema/form)
- [`Heading`](https://stencila.ghost.io/docs/reference/schema/heading)
- [`IfBlock`](https://stencila.ghost.io/docs/reference/schema/if-block)
- [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)
- [`IncludeBlock`](https://stencila.ghost.io/docs/reference/schema/include-block)
- [`InlinesBlock`](https://stencila.ghost.io/docs/reference/schema/inlines-block)
- [`InstructionBlock`](https://stencila.ghost.io/docs/reference/schema/instruction-block)
- [`List`](https://stencila.ghost.io/docs/reference/schema/list)
- [`MathBlock`](https://stencila.ghost.io/docs/reference/schema/math-block)
- [`Paragraph`](https://stencila.ghost.io/docs/reference/schema/paragraph)
- [`PromptBlock`](https://stencila.ghost.io/docs/reference/schema/prompt-block)
- [`QuoteBlock`](https://stencila.ghost.io/docs/reference/schema/quote-block)
- [`RawBlock`](https://stencila.ghost.io/docs/reference/schema/raw-block)
- [`Section`](https://stencila.ghost.io/docs/reference/schema/section)
- [`StyledBlock`](https://stencila.ghost.io/docs/reference/schema/styled-block)
- [`SuggestionBlock`](https://stencila.ghost.io/docs/reference/schema/suggestion-block)
- [`Table`](https://stencila.ghost.io/docs/reference/schema/table)
- [`ThematicBreak`](https://stencila.ghost.io/docs/reference/schema/thematic-break)
- [`VideoObject`](https://stencila.ghost.io/docs/reference/schema/video-object)
- [`Walkthrough`](https://stencila.ghost.io/docs/reference/schema/walkthrough)

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
| `PromptBlock`      | Min+       | Do not generate `PromptBlock` nodes in block content.      | -                 |
| `Section`          | Min+       | Do not generate `Section` nodes in block content.          | -                 |
|                    | Low+       | Generate `Section` nodes in block content.                 | Default for level |
| `SuggestionBlock`  | Min+       | Do not generate `SuggestionBlock` nodes in block content.  | -                 |
| `VideoObject`      | Min+       | Do not generate `VideoObject` nodes in block content.      | -                 |
| `Walkthrough`      | Min+       | Do not generate `Walkthrough` nodes in block content.      | -                 |

# Source

This documentation was generated from [`Block.yaml`](https://github.com/stencila/stencila/blob/main/schema/Block.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
