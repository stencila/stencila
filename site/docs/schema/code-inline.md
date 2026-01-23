---
title: Code Inline
description: Inline code.
---

# Properties

The `CodeInline` type has these properties:

| Name                  | Description                              | Type                                        | Inherited from                   |
| --------------------- | ---------------------------------------- | ------------------------------------------- | -------------------------------- |
| `id`                  | The identifier for this item.            | [`String`](./string.md)                     | [`Entity`](./entity.md)          |
| `code`                | The code.                                | [`Cord`](./cord.md)                         | [`CodeStatic`](./code-static.md) |
| `programmingLanguage` | The programming language of the code.    | [`String`](./string.md)                     | [`CodeStatic`](./code-static.md) |
| `authors`             | The authors of the code.                 | [`Author`](./author.md)*                    | [`CodeStatic`](./code-static.md) |
| `provenance`          | A summary of the provenance of the code. | [`ProvenanceCount`](./provenance-count.md)* | [`CodeStatic`](./code-static.md) |

# Related

The `CodeInline` type is related to these types:

- Parents: [`CodeStatic`](./code-static.md)
- Children: none

# Bindings

The `CodeInline` type is represented in:

- [JSON-LD](https://stencila.org/CodeInline.jsonld)
- [JSON Schema](https://stencila.org/CodeInline.schema.json)
- Python class [`CodeInline`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`CodeInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_inline.rs)
- TypeScript class [`CodeInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeInline.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `CodeInline` type are generated using the following strategies.

::: table

| Property              | Complexity | Description                                                                                                                    | Strategy                                      |
| --------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------- |
| `code`                | Min+       | Generate a simple fixed string of code.                                                                                        | `Cord::from("code")`                          |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which<br><br>can be problematic in Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`   |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                               | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`     |
|                       | Max        | Generate an arbitrary string.                                                                                                  | `String::arbitrary().prop_map(Cord::from)`    |
| `programmingLanguage` | Min+       | Do not generate a programming language.                                                                                        | `None`                                        |
|                       | Low+       | Generate one of the well known programming language short names.                                                               | `option::of(r"(cpp)\|(js)\|(py)\|(r)\|(ts)")` |
|                       | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                  | `option::of(r"[a-zA-Z0-9]{1,10}")`            |
|                       | Max        | Generate an arbitrary string.                                                                                                  | `option::of(String::arbitrary())`             |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`CodeInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
