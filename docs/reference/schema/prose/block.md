# Block

**Union type in block content node types.**

**`@id`**: `stencila:Block`

## Members

The `Block` type has these members:

- [`Admonition`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/admonition.md)
- [`Call`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call.md)
- [`Claim`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/claim.md)
- [`CodeBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-block.md)
- [`CodeChunk`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-chunk.md)
- [`Division`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/division.md)
- [`Figure`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/figure.md)
- [`For`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/for.md)
- [`Form`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form.md)
- [`Heading`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md)
- [`If`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if.md)
- [`Include`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include.md)
- [`List`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md)
- [`MathBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math-block.md)
- [`Paragraph`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md)
- [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote-block.md)
- [`Section`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section.md)
- [`Table`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table.md)
- [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/thematic-break.md)

## Bindings

The `Block` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Block.jsonld)
- [JSON Schema](https://stencila.dev/Block.schema.json)
- Python type [`Block`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/block.py)
- Rust type [`Block`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/block.rs)
- TypeScript type [`Block`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Block.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `Block` type are generated using the following strategies[^1] for each complexity level. Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

| Variant      | Complexity | Description                                          | Strategy          |
| ------------ | ---------- | ---------------------------------------------------- | ----------------- |
| `Admonition` | Min+       | Do not generate `Admonition` nodes in block content. | -                 |
|              | Low+       | Generate `Admonition` nodes in block content.        | Default for level |
| `Call`       | Min+       | Do not generate `Call` nodes in block content.       | -                 |
|              | Low+       | Generate `Call` nodes in block content.              | Default for level |
| `Claim`      | Min+       | Do not generate `Claim` nodes in block content.      | -                 |
|              | Low+       | Generate `Claim` nodes in block content.             | Default for level |
| `CodeChunk`  | Min+       | Do not generate `CodeChunk` nodes in block content.  | -                 |
|              | Low+       | Generate `CodeChunk` nodes in block content.         | Default for level |
| `Figure`     | Min+       | Do not generate `Figure` nodes in block content.     | -                 |
|              | Low+       | Generate `Figure` nodes in block content.            | Default for level |
| `For`        | Min+       | Do not generate `For` nodes in block content.        | -                 |
|              | Low+       | Generate `For` nodes in block content.               | Default for level |
| `Form`       | Min+       | Do not generate `Form` nodes in block content.       | -                 |
| `If`         | Min+       | Do not generate `If` nodes in block content.         | -                 |
|              | Low+       | Generate `If` nodes in block content.                | Default for level |
| `Include`    | Min+       | Do not generate `Include` nodes in block content.    | -                 |
|              | Low+       | Generate `Include` nodes in block content.           | Default for level |
| `Section`    | Min+       | Do not generate `Section` nodes in block content.    | -                 |
|              | Low+       | Generate `Section` nodes in block content.           | Default for level |

## Source

This documentation was generated from [`Block.yaml`](https://github.com/stencila/stencila/blob/main/schema/Block.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.