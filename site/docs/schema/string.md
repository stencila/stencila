---
title: String
description: A value comprised of a string of characters.
---

This is an implementation of schema.org [`Text`](https://schema.org/Text),
exposed in Stencila Schema as `String` for consistency with common programming
and data models.

Stencila uses a dedicated primitive node so string values can participate in
validation, serialization, and data-oriented workflows alongside other
primitive node types. The rename emphasizes its role as a primitive value
rather than rich inline text content.

See also [`Cord`](./cord.md) for character-sequence content and
[`Primitive`](./primitive.md) for the enclosing union.


# Analogues

The following external types, elements, or nodes are similar to a `String`:

- schema.org [`Text`](https://schema.org/Text): Direct schema.org source type, renamed in Stencila to emphasize primitive string semantics rather than rich text-node content.
- [JSON string](https://www.json.org/json-en.html)

# Bindings

The `String` type is represented in:

- [JSON-LD](https://stencila.org/String.jsonld)
- [JSON Schema](https://stencila.org/String.schema.json)
- Python type [`String`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`String`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/string.rs)
- TypeScript type [`String`](https://github.com/stencila/stencila/blob/main/ts/src/types/String.ts)

***

This documentation was generated from [`String.yaml`](https://github.com/stencila/stencila/blob/main/schema/String.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
