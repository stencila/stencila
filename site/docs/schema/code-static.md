---
title: Code Static
description: Abstract base type for non-executable code nodes (e.g. `CodeBlock`).
---

The most important, and only required, property of a `CodeStatic` node is `code`, a `string` of the source code.
There are no restrictions on the length or content of `code` and it is possible for it to be syntactically
invalid for the specified `programmingLanguage`.


# Properties

The `CodeStatic` type has these properties:

| Name                  | Description                              | Type                                        | Inherited from          | `JSON-LD @id`                                                          | Aliases                                        |
| --------------------- | ---------------------------------------- | ------------------------------------------- | ----------------------- | ---------------------------------------------------------------------- | ---------------------------------------------- |
| `id`                  | The identifier for this item.            | [`String`](./string.md)                     | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                                   | -                                              |
| `code`                | The code.                                | [`Cord`](./cord.md)                         | -                       | `stencila:code`                                                        | -                                              |
| `programmingLanguage` | The programming language of the code.    | [`String`](./string.md)                     | -                       | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language` |
| `authors`             | The authors of the code.                 | [`Author`](./author.md)*                    | -                       | [`schema:author`](https://schema.org/author)                           | `author`                                       |
| `provenance`          | A summary of the provenance of the code. | [`ProvenanceCount`](./provenance-count.md)* | -                       | `stencila:provenance`                                                  | -                                              |

# Related

The `CodeStatic` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`CodeBlock`](./code-block.md), [`CodeInline`](./code-inline.md)

# Bindings

The `CodeStatic` type is represented in:

- [JSON-LD](https://stencila.org/CodeStatic.jsonld)
- [JSON Schema](https://stencila.org/CodeStatic.schema.json)
- Python class [`CodeStatic`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_static.py)
- Rust struct [`CodeStatic`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_static.rs)
- TypeScript class [`CodeStatic`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeStatic.ts)

# Source

This documentation was generated from [`CodeStatic.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeStatic.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
