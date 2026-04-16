---
title: String Hint
description: A concise summary of the properties of a `String`.
---

This is a type used in Stencila Schema for providing a concise summary of the
properties of a [`String`](./string.md).

It exists to support both human and machine understanding of textual data,
including schema inference, editing assistance, and downstream code
generation. Rather than turning observations into hard constraints, it
summarizes characteristics that can help tools and readers work with string
values.

Key properties describe expected patterns, length characteristics, and
representative values.


# Properties

The `StringHint` type has these properties:

| Name    | Description                             | Type                      | Inherited from          |
| ------- | --------------------------------------- | ------------------------- | ----------------------- |
| `chars` | The number of characters in the string. | [`Integer`](./integer.md) | -                       |
| `id`    | The identifier for this item.           | [`String`](./string.md)   | [`Entity`](./entity.md) |

# Related

The `StringHint` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `StringHint` type is represented in:

- [JSON-LD](https://stencila.org/StringHint.jsonld)
- [JSON Schema](https://stencila.org/StringHint.schema.json)
- Python class [`StringHint`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`StringHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/string_hint.rs)
- TypeScript class [`StringHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/StringHint.ts)

***

This documentation was generated from [`StringHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/StringHint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
