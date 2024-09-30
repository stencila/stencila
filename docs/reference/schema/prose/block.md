# Block

**Union type in block content node types.**

**`@id`**: `stencila:Block`

## Members

The `Block` type has these members:

- [`Admonition`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/admonition.md)
- [`CallBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call-block.md)
- [`Claim`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/claim.md)
- [`CodeBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-block.md)
- [`CodeChunk`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-chunk.md)
- [`DeleteBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/delete-block.md)
- [`Figure`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/figure.md)
- [`ForBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/for-block.md)
- [`Form`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form.md)
- [`Heading`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md)
- [`IfBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if-block.md)
- [`IncludeBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include-block.md)
- [`InsertBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/insert-block.md)
- [`InstructionBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-block.md)
- [`List`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md)
- [`MathBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math-block.md)
- [`ModifyBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-block.md)
- [`Paragraph`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md)
- [`PromptBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/prompt-block.md)
- [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote-block.md)
- [`RawBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/raw-block.md)
- [`ReplaceBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/replace-block.md)
- [`Section`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section.md)
- [`StyledBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled-block.md)
- [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-block.md)
- [`Table`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table.md)
- [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/thematic-break.md)

## Bindings

The `Block` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Block.jsonld)
- [JSON Schema](https://stencila.org/Block.schema.json)
- Python type [`Block`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/block.py)
- Rust type [`Block`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/block.rs)
- TypeScript type [`Block`](https://github.com/stencila/stencila/blob/main/ts/src/types/Block.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `Block` type are generated using the following strategies[^1] for each complexity level. Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

| Variant            | Complexity | Description                                                | Strategy          |
| ------------------ | ---------- | ---------------------------------------------------------- | ----------------- |
| `Admonition`       | Min+       | Do not generate `Admonition` nodes in block content.       | -                 |
|                    | Low+       | Generate `Admonition` nodes in block content.              | Default for level |
| `CallBlock`        | Min+       | Do not generate `CallBlock` nodes in block content.        | -                 |
|                    | Low+       | Generate `CallBlock` nodes in block content.               | Default for level |
| `Claim`            | Min+       | Do not generate `Claim` nodes in block content.            | -                 |
|                    | Low+       | Generate `Claim` nodes in block content.                   | Default for level |
| `CodeChunk`        | Min+       | Do not generate `CodeChunk` nodes in block content.        | -                 |
|                    | Low+       | Generate `CodeChunk` nodes in block content.               | Default for level |
| `DeleteBlock`      | Min+       | Do not generate `DeleteBlock` nodes in block content.      | -                 |
| `Figure`           | Min+       | Do not generate `Figure` nodes in block content.           | -                 |
|                    | Low+       | Generate `Figure` nodes in block content.                  | Default for level |
| `ForBlock`         | Min+       | Do not generate `ForBlock` nodes in block content.         | -                 |
|                    | Low+       | Generate `ForBlock` nodes in block content.                | Default for level |
| `Form`             | Min+       | Do not generate `Form` nodes in block content.             | -                 |
| `IfBlock`          | Min+       | Do not generate `IfBlock` nodes in block content.          | -                 |
|                    | Low+       | Generate `IfBlock` nodes in block content.                 | Default for level |
| `IncludeBlock`     | Min+       | Do not generate `IncludeBlock` nodes in block content.     | -                 |
|                    | Low+       | Generate `IncludeBlock` nodes in block content.            | Default for level |
| `InsertBlock`      | Min+       | Do not generate `InsertBlock` nodes in block content.      | -                 |
| `InstructionBlock` | Min+       | Do not generate `InstructionBlock` nodes in block content. | -                 |
| `ModifyBlock`      | Min+       | Do not generate `ModifyBlock` nodes in block content.      | -                 |
| `PromptBlock`      | Min+       | Do not generate `PromptBlock` nodes in block content.      | -                 |
| `ReplaceBlock`     | Min+       | Do not generate `ReplaceBlock` nodes in block content.     | -                 |
| `Section`          | Min+       | Do not generate `Section` nodes in block content.          | -                 |
|                    | Low+       | Generate `Section` nodes in block content.                 | Default for level |
| `SuggestionBlock`  | Min+       | Do not generate `SuggestionBlock` nodes in block content.  | -                 |

## Source

This documentation was generated from [`Block.yaml`](https://github.com/stencila/stencila/blob/main/schema/Block.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
