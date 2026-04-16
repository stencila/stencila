---
title: Unsigned Integer
description: An integer value that is greater or equal to zero.
---

This is a primitive type used in Stencila Schema for integers greater than or equal
to zero.

It exists because non-negative counts, indices, and similar quantities occur
frequently in documents and execution metadata, and having a distinct type
lets Stencila express that constraint directly in the schema.

See also [`Integer`](./integer.md) and properties such as execution counts and
active-suggestion indices that use this type.


# Bindings

The `UnsignedInteger` type is represented in:

- [JSON-LD](https://stencila.org/UnsignedInteger.jsonld)
- [JSON Schema](https://stencila.org/UnsignedInteger.schema.json)
- Python type [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/unsigned_integer.rs)
- TypeScript type [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/ts/src/types/UnsignedInteger.ts)

***

This documentation was generated from [`UnsignedInteger.yaml`](https://github.com/stencila/stencila/blob/main/schema/UnsignedInteger.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
