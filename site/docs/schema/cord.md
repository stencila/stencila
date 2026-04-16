---
title: Cord
description: A CRDT-backed sequence of characters.
---

This is a string-like type used in Stencila Schema for collaboratively
editable text.

It exists to distinguish ordinary replacement-based string values from text
stored and synchronized as a CRDT, where changes are modeled as insertions and
deletions. This makes it suitable for collaborative editing and fine-grained
text synchronization.

Common uses include the `value` property of [`Text`](./text.md) and the `code`
property of executable code nodes.


# Bindings

The `Cord` type is represented in:

- [JSON-LD](https://stencila.org/Cord.jsonld)
- [JSON Schema](https://stencila.org/Cord.schema.json)
- Python type [`Cord`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Cord`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/cord.rs)
- TypeScript type [`Cord`](https://github.com/stencila/stencila/blob/main/ts/src/types/Cord.ts)

***

This documentation was generated from [`Cord.yaml`](https://github.com/stencila/stencila/blob/main/schema/Cord.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
