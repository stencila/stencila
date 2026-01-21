---
title: Object Hint
description: A hint to the structure of an `Object`.
---

# Properties

The `ObjectHint` type has these properties:

| Name     | Description                                  | Type                      | Inherited from          |
| -------- | -------------------------------------------- | ------------------------- | ----------------------- |
| `id`     | The identifier for this item.                | [`String`](./string.md)   | [`Entity`](./entity.md) |
| `length` | The number of entries in the object.         | [`Integer`](./integer.md) | -                       |
| `keys`   | The keys of the object's entries.            | [`String`](./string.md)*  | -                       |
| `values` | Hints to the values of the object's entries. | [`Hint`](./hint.md)*      | -                       |

# Related

The `ObjectHint` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ObjectHint` type is represented in:

- [JSON-LD](https://stencila.org/ObjectHint.jsonld)
- [JSON Schema](https://stencila.org/ObjectHint.schema.json)
- Python class [`ObjectHint`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/object_hint.py)
- Rust struct [`ObjectHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/object_hint.rs)
- TypeScript class [`ObjectHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/ObjectHint.ts)

# Source

This documentation was generated from [`ObjectHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ObjectHint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
