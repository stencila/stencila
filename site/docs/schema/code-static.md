---
title: Code Static
description: An abstract base type for non-executable code nodes.
---

This is an abstract base type used in Stencila Schema for non-executable code nodes.

It exists to represent code as document content without normal execution
semantics, while still preserving source text, programming language,
authorship, and provenance metadata. This provides a common foundation for
static code blocks and inline code.

Key properties include `code`, `programmingLanguage`, `authors`, and
`provenance`.


# Properties

The `CodeStatic` type has these properties:

| Name                  | Description                              | Type                                        | Inherited from          |
| --------------------- | ---------------------------------------- | ------------------------------------------- | ----------------------- |
| `code`                | The code.                                | [`Cord`](./cord.md)                         | -                       |
| `programmingLanguage` | The programming language of the code.    | [`String`](./string.md)                     | -                       |
| `authors`             | The authors of the code.                 | [`Author`](./author.md)*                    | -                       |
| `provenance`          | A summary of the provenance of the code. | [`ProvenanceCount`](./provenance-count.md)* | -                       |
| `id`                  | The identifier for this item.            | [`String`](./string.md)                     | [`Entity`](./entity.md) |

# Related

The `CodeStatic` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`CodeBlock`](./code-block.md), [`CodeInline`](./code-inline.md)

# Bindings

The `CodeStatic` type is represented in:

- [JSON-LD](https://stencila.org/CodeStatic.jsonld)
- [JSON Schema](https://stencila.org/CodeStatic.schema.json)
- Python class [`CodeStatic`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`CodeStatic`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_static.rs)
- TypeScript class [`CodeStatic`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeStatic.ts)

***

This documentation was generated from [`CodeStatic.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeStatic.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
