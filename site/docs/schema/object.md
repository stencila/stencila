---
title: Object
description: An object value.
---

This is a primitive type used in Stencila Schema for object values.

It exists so keyed collections of primitive values can participate in the same
node-based data model as scalar and array primitives. Keys are strings and
values are restricted to [`Primitive`](./primitive.md) values, including
nested `Object` and `Array` values.

See also [`Primitive`](./primitive.md) for the broader primitive-value union.


# Analogues

The following external types, elements, or nodes are similar to a `Object`:

- [JSON object](https://www.json.org/json-en.html): Direct structural analogue for string-keyed primitive mappings.

# Bindings

The `Object` type is represented in:

- [JSON-LD](https://stencila.org/Object.jsonld)
- [JSON Schema](https://stencila.org/Object.schema.json)
- Python type [`Object`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Object`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/object.rs)
- TypeScript type [`Object`](https://github.com/stencila/stencila/blob/main/ts/src/types/Object.ts)

***

This documentation was generated from [`Object.yaml`](https://github.com/stencila/stencila/blob/main/schema/Object.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
