---
title: Primitive
description: A union type for primitive values.
---

This is a union type used in Stencila Schema for primitive data values.

It is broadly analogous to JSON values, but uses Stencila's explicit primitive
node types and includes both `Integer` and `UnsignedInteger` alongside
`Array`, `Object`, `String`, `Number`, `Boolean`, and `Null`.

Temporal node types such as [`Date`](./date.md), [`Time`](./time.md), and
[`DateTime`](./date-time.md) are excluded because Stencila models them as
distinct typed nodes rather than primitive strings.


# Members

The `Primitive` type has these members:

- [`Null`](./null.md)
- [`Boolean`](./boolean.md)
- [`Integer`](./integer.md)
- [`UnsignedInteger`](./unsigned-integer.md)
- [`Number`](./number.md)
- [`String`](./string.md)
- [`Array`](./array.md)
- [`Object`](./object.md)

# Bindings

The `Primitive` type is represented in:

- [JSON-LD](https://stencila.org/Primitive.jsonld)
- [JSON Schema](https://stencila.org/Primitive.schema.json)
- Python type [`Primitive`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Primitive`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/primitive.rs)
- TypeScript type [`Primitive`](https://github.com/stencila/stencila/blob/main/ts/src/types/Primitive.ts)

***

This documentation was generated from [`Primitive.yaml`](https://github.com/stencila/stencila/blob/main/schema/Primitive.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
