---
title: Object Hint
description: A concise summary of the structure of an `Object`.
---

This is a type used in Stencila Schema for providing a concise summary of the
structure of an [`Object`](./object.md).

It exists to support both human and machine understanding of keyed data,
including schema inference, editing assistance, and code generation
workflows. Rather than imposing hard validation constraints, it summarizes
observed or inferred characteristics that can help tools and readers work with
object-shaped data.

Key properties describe expected keys, per-key hints, and overall object
characteristics.


# Properties

The `ObjectHint` type has these properties:

| Name     | Description                                  | Type                      | Inherited from          |
| -------- | -------------------------------------------- | ------------------------- | ----------------------- |
| `length` | The number of entries in the object.         | [`Integer`](./integer.md) | -                       |
| `keys`   | The keys of the object's entries.            | [`String`](./string.md)*  | -                       |
| `values` | Hints to the values of the object's entries. | [`Hint`](./hint.md)*      | -                       |
| `id`     | The identifier for this item.                | [`String`](./string.md)   | [`Entity`](./entity.md) |

# Related

The `ObjectHint` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ObjectHint` type is represented in:

- [JSON-LD](https://stencila.org/ObjectHint.jsonld)
- [JSON Schema](https://stencila.org/ObjectHint.schema.json)
- Python class [`ObjectHint`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ObjectHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/object_hint.rs)
- TypeScript class [`ObjectHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/ObjectHint.ts)

***

This documentation was generated from [`ObjectHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ObjectHint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
