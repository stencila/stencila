---
title: Unknown
description: A placeholder for a value of unknown type.
---

This is a fallback type used in Stencila Schema for values or nodes whose concrete
type is not known.

It exists to preserve information during decoding, transformation, or partial
validation when content cannot yet be mapped to a more specific schema type.
This helps Stencila avoid unnecessary data loss while still keeping the
uncertainty explicit.

This type is mainly useful at interoperability and ingestion boundaries.


# Properties

The `Unknown` type has these properties:

| Name | Description                   | Type                    | Inherited from          |
| ---- | ----------------------------- | ----------------------- | ----------------------- |
| `id` | The identifier for this item. | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `Unknown` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Unknown` type is represented in:

- [JSON-LD](https://stencila.org/Unknown.jsonld)
- [JSON Schema](https://stencila.org/Unknown.schema.json)
- Python class [`Unknown`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Unknown`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/unknown.rs)
- TypeScript class [`Unknown`](https://github.com/stencila/stencila/blob/main/ts/src/types/Unknown.ts)

***

This documentation was generated from [`Unknown.yaml`](https://github.com/stencila/stencila/blob/main/schema/Unknown.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
