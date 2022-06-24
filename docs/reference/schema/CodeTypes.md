# Code Types

**All type schemas that are derived from Code**

This schema type is marked as **experimental** 🧪 and is subject to change.

## Members

| `@id`                                                                     | Type                                | Description                                                                                        |
| ------------------------------------------------------------------------- | ----------------------------------- | -------------------------------------------------------------------------------------------------- |
| [stencila:Code](https://schema.stenci.la/Code.jsonld)                     | [Code](Code.md)                     | Base type for non-executable (e.g. `CodeBlock`) and executable (e.g. `CodeExpression`) code nodes. |
| [stencila:CodeBlock](https://schema.stenci.la/CodeBlock.jsonld)           | [CodeBlock](CodeBlock.md)           | A code block.                                                                                      |
| [stencila:CodeChunk](https://schema.stenci.la/CodeChunk.jsonld)           | [CodeChunk](CodeChunk.md)           | A executable chunk of code.                                                                        |
| [stencila:CodeExecutable](https://schema.stenci.la/CodeExecutable.jsonld) | [CodeExecutable](CodeExecutable.md) | Base type for executable code nodes (i.e. `CodeChunk` and `CodeExpression`).                       |
| [stencila:CodeExpression](https://schema.stenci.la/CodeExpression.jsonld) | [CodeExpression](CodeExpression.md) | An executable programming code expression.                                                         |
| [stencila:CodeFragment](https://schema.stenci.la/CodeFragment.jsonld)     | [CodeFragment](CodeFragment.md)     | Inline code.                                                                                       |

## Available as

- [JSON-LD](https://schema.stenci.la/stencila.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/CodeTypes.schema.json)
