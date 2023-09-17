---
title:
- type: Text
  value: CodeStatic
---

# Code Static

**Abstract base type for non-executable code nodes (e.g. `CodeBlock`).**

The most important, and only required, property of a `CodeStatic` node is `code`, a `string` of the source code.
There are no restrictions on the length or content of `code` and it is possible for it to be syntactically
invalid for the specified `programmingLanguage`.


**`@id`**: `stencila:CodeStatic`

## Properties

The `CodeStatic` type has these properties:

| Name                | `@id`                                                                  | Type                                                               | Description                           | Inherited from                                                              |
| ------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------- | --------------------------------------------------------------------------- |
| id                  | [`schema:id`](https://schema.org/id)                                   | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item          | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)         |
| code                | `stencila:code`                                                        | [`Cord`](https://stencila.dev/docs/reference/schema/data/cord)     | The code.                             | [`CodeStatic`](https://stencila.dev/docs/reference/schema/code/code-static) |
| programmingLanguage | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The programming language of the code. | [`CodeStatic`](https://stencila.dev/docs/reference/schema/code/code-static) |

## Related

The `CodeStatic` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: [`CodeBlock`](https://stencila.dev/docs/reference/schema/code/code-block), [`CodeFragment`](https://stencila.dev/docs/reference/schema/code/code-fragment)

## Bindings

The `CodeStatic` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/CodeStatic.jsonld)
- [JSON Schema](https://stencila.dev/CodeStatic.schema.json)
- Python class [`CodeStatic`](https://github.com/stencila/stencila/blob/main/python/stencila/types/code_static.py)
- Rust struct [`CodeStatic`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_static.rs)
- TypeScript class [`CodeStatic`](https://github.com/stencila/stencila/blob/main/typescript/src/types/CodeStatic.ts)

## Source

This documentation was generated from [`CodeStatic.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeStatic.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).