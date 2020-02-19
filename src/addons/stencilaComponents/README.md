# Stencila Web Components addon

Adds Stencila's [Web Components](https://github.com/stencila/designa/tree/master/packages/components) to provide interactivity to certain types of document nodes.

Currently the following node types have Web Components:

| Node type        | Custom element               |
| ---------------- | ---------------------------- |
| `CodeChunk`      | `<stencila-code-chunk>`      |
| `CodeExpression` | `<stencila-code-expression>` |

## Notes

- We anticipate that more components will be added over time, for other document nodes, to provide greater interactivity in documents.

- Theme authors should be able to override the styles of the web components as part of their theme.
