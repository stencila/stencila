---
title:
- type: Text
  value: Primitive
---

# Primitive

**Union type for all primitives values**

Analogous to JSON values but adds `Integer` and `UnsignedInteger`.

Similar to https://schema.org/DataType "The basic data types such as Integers, Strings, etc."
but includes `Array` and `Object` and excludes `Date`, `Time` and `DateTime` (which are
treated in this schema as `Entity`s having a `type` property to disambiguate them from strings).


**`@id`**: `stencila:Primitive`

## Members

The `Primitive` type has these members:

- [`Null`](https://stencila.dev/docs/reference/schema/data/null)
- [`Boolean`](https://stencila.dev/docs/reference/schema/data/boolean)
- [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)
- [`UnsignedInteger`](https://stencila.dev/docs/reference/schema/data/unsigned-integer)
- [`Number`](https://stencila.dev/docs/reference/schema/data/number)
- [`String`](https://stencila.dev/docs/reference/schema/data/string)
- [`Array`](https://stencila.dev/docs/reference/schema/data/array)
- [`Object`](https://stencila.dev/docs/reference/schema/data/object)

## Bindings

The `Primitive` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Primitive.jsonld)
- [JSON Schema](https://stencila.dev/Primitive.schema.json)
- Python type [`Primitive`](https://github.com/stencila/stencila/blob/main/python/stencila/types/primitive.py)
- Rust type [`Primitive`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/primitive.rs)
- TypeScript type [`Primitive`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Primitive.ts)

## Source

This documentation was generated from [`Primitive.yaml`](https://github.com/stencila/stencila/blob/main/schema/Primitive.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).