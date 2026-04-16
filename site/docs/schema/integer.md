---
title: Integer
description: An integer value.
---

This is an implementation of schema.org
[`Integer`](https://schema.org/Integer).

In Stencila Schema it is represented as a primitive node so integer values can
be typed, validated, and serialized consistently across document metadata,
parameters, and data structures.

See also [`Number`](./number.md) and
[`UnsignedInteger`](./unsigned-integer.md).


# Analogues

The following external types, elements, or nodes are similar to a `Integer`:

- schema.org [`Integer`](https://schema.org/Integer)
- [JSON integer-constrained number](https://json-schema.org/understanding-json-schema/reference/numeric): Approximate analogue because JSON itself has only numbers, while integer-ness is usually enforced by JSON Schema or similar validators.

# Bindings

The `Integer` type is represented in:

- [JSON-LD](https://stencila.org/Integer.jsonld)
- [JSON Schema](https://stencila.org/Integer.schema.json)
- Python type [`Integer`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Integer`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer.rs)
- TypeScript type [`Integer`](https://github.com/stencila/stencila/blob/main/ts/src/types/Integer.ts)

***

This documentation was generated from [`Integer.yaml`](https://github.com/stencila/stencila/blob/main/schema/Integer.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
