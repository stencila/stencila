---
title: 'Null'
description: The null value.
---

This is a primitive type used in Stencila Schema for the null value.

It exists so absence can be represented explicitly within the same node system
used for other primitive values, which is useful for structured data,
validation, and serialization.

See also [`Primitive`](./primitive.md) and the validator types that can reason
about null values.


# Analogues

The following external types, elements, or nodes are similar to a `Null`:

- [JSON null](https://www.json.org/json-en.html)

# Bindings

The `Null` type is represented in:

- [JSON-LD](https://stencila.org/Null.jsonld)
- [JSON Schema](https://stencila.org/Null.schema.json)
- Python type [`Null`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Null`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/null.rs)
- TypeScript type [`Null`](https://github.com/stencila/stencila/blob/main/ts/src/types/Null.ts)

***

This documentation was generated from [`Null.yaml`](https://github.com/stencila/stencila/blob/main/schema/Null.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
