# Default components

The [default set of Web Components](https://github.com/stencila/designa/tree/master/packages/components) to provide interactivity to document nodes.

Currently, the following node types have Web Components. Encoda will encode these nodes types as custom HTML elements that get hydrated into these components.

| Node type        | Custom element               |
| ---------------- | ---------------------------- |
| `CodeChunk`      | `<stencila-code-chunk>`      |
| `CodeExpression` | `<stencila-code-expression>` |

More components will be added over time. In the meantime, the "pseudo-components" in sibling folders to this one, provide styling for some other node types.

## Notes

- Theme authors should be able to override the styles of the web components as part of their theme.
