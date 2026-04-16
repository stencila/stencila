---
title: Array
description: An array value.
---

This is a primitive type used in Stencila Schema for array values.

It exists so ordered collections of primitive values can participate in the
same node-based data model as other scalar and structured primitive types.
Array items are restricted to [`Primitive`](./primitive.md) values, including
nested `Array` and `Object` values.

See also [`TupleValidator`](./tuple-validator.md) and
[`ArrayValidator`](./array-validator.md) for array-specific constraints.


# Analogues

The following external types, elements, or nodes are similar to a `Array`:

- [JSON array](https://www.json.org/json-en.html): Direct structural analogue for ordered primitive collections.

# Bindings

The `Array` type is represented in:

- [JSON-LD](https://stencila.org/Array.jsonld)
- [JSON Schema](https://stencila.org/Array.schema.json)
- Python type [`Array`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Array`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/array.rs)
- TypeScript type [`Array`](https://github.com/stencila/stencila/blob/main/ts/src/types/Array.ts)

***

This documentation was generated from [`Array.yaml`](https://github.com/stencila/stencila/blob/main/schema/Array.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
