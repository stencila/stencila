# Code Static

**Abstract base type for non-executable code nodes (e.g. `CodeBlock`).**

The most important, and only required, property of a `CodeStatic` node is `code`, a `string` of the source code.
There are no restrictions on the length or content of `code` and it is possible for it to be syntactically
invalid for the specified `programmingLanguage`.


**`@id`**: `stencila:CodeStatic`

## Properties

The `CodeStatic` type has these properties:

| Name                  | Aliases                                        | `@id`                                                                  | Type                                                                                            | Description                           | Inherited from                                                                                   |
| --------------------- | ---------------------------------------------- | ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                  | -                                              | [`schema:id`](https://schema.org/id)                                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `code`                | -                                              | `stencila:code`                                                        | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)     | The code.                             | -                                                                                                |
| `programmingLanguage` | `programming-language`, `programming_language` | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The programming language of the code. | -                                                                                                |

## Related

The `CodeStatic` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`CodeBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-block.md), [`CodeInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-inline.md)

## Bindings

The `CodeStatic` type is represented in these bindings:

- [JSON-LD](https://stencila.org/CodeStatic.jsonld)
- [JSON Schema](https://stencila.org/CodeStatic.schema.json)
- Python class [`CodeStatic`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_static.py)
- Rust struct [`CodeStatic`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_static.rs)
- TypeScript class [`CodeStatic`](https://github.com/stencila/stencila/blob/main/typescript/src/types/CodeStatic.ts)

## Source

This documentation was generated from [`CodeStatic.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeStatic.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).