---
title: Primitive
description: Union type for all primitives values.
config:
  publish:
    ghost:
      type: post
      slug: primitive
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

Analogous to JSON values but adds `Integer` and `UnsignedInteger`.

Similar to https://schema.org/DataType "The basic data types such as Integers, Strings, etc."
but includes `Array` and `Object` and excludes `Date`, `Time` and `DateTime` (which are
treated in this schema as `Entity`s having a `type` property to disambiguate them from strings).


# Members

The `Primitive` type has these members:

- [`Null`](https://stencila.ghost.io/docs/reference/schema/null)
- [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)
- [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)
- [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer)
- [`Number`](https://stencila.ghost.io/docs/reference/schema/number)
- [`String`](https://stencila.ghost.io/docs/reference/schema/string)
- [`Array`](https://stencila.ghost.io/docs/reference/schema/array)
- [`Object`](https://stencila.ghost.io/docs/reference/schema/object)

# Bindings

The `Primitive` type is represented in:

- [JSON-LD](https://stencila.org/Primitive.jsonld)
- [JSON Schema](https://stencila.org/Primitive.schema.json)
- Python type [`Primitive`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/primitive.py)
- Rust type [`Primitive`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/primitive.rs)
- TypeScript type [`Primitive`](https://github.com/stencila/stencila/blob/main/ts/src/types/Primitive.ts)

# Source

This documentation was generated from [`Primitive.yaml`](https://github.com/stencila/stencila/blob/main/schema/Primitive.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
