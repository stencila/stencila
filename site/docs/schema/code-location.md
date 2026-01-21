---
title: Code Location
description: The location within some source code.
---

# Properties

The `CodeLocation` type has these properties:

| Name          | Description                                                        | Type                                       | Inherited from          |
| ------------- | ------------------------------------------------------------------ | ------------------------------------------ | ----------------------- |
| `id`          | The identifier for this item.                                      | [`String`](./string.md)                    | [`Entity`](./entity.md) |
| `source`      | The source of the code, a file path, label or URL.                 | [`String`](./string.md)                    | -                       |
| `startLine`   | The 0-based index if the first line on which the error occurred.   | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `startColumn` | The 0-based index if the first column on which the error occurred. | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `endLine`     | The 0-based index if the last line on which the error occurred.    | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `endColumn`   | The 0-based index if the last column on which the error occurred.  | [`UnsignedInteger`](./unsigned-integer.md) | -                       |

# Related

The `CodeLocation` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `CodeLocation` type is represented in:

- [JSON-LD](https://stencila.org/CodeLocation.jsonld)
- [JSON Schema](https://stencila.org/CodeLocation.schema.json)
- Python class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_location.py)
- Rust struct [`CodeLocation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_location.rs)
- TypeScript class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeLocation.ts)

# Source

This documentation was generated from [`CodeLocation.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeLocation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
