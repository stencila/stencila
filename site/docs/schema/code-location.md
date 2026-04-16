---
title: Code Location
description: A location within source code.
---

This is a type used in Stencila Schema for pointing to a location within source
code.

It exists to support diagnostics, references, provenance, and tooling
integrations that need to connect document nodes back to specific code spans.
By representing locations structurally, Stencila can preserve this information
across serialization and execution workflows.

Key properties identify the source, range, and positional information for the
code span.


# Properties

The `CodeLocation` type has these properties:

| Name          | Description                                                        | Type                                       | Inherited from          |
| ------------- | ------------------------------------------------------------------ | ------------------------------------------ | ----------------------- |
| `source`      | The source of the code, a file path, label or URL.                 | [`String`](./string.md)                    | -                       |
| `startLine`   | The 0-based index if the first line on which the error occurred.   | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `startColumn` | The 0-based index if the first column on which the error occurred. | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `endLine`     | The 0-based index if the last line on which the error occurred.    | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `endColumn`   | The 0-based index if the last column on which the error occurred.  | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `id`          | The identifier for this item.                                      | [`String`](./string.md)                    | [`Entity`](./entity.md) |

# Related

The `CodeLocation` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `CodeLocation` type is represented in:

- [JSON-LD](https://stencila.org/CodeLocation.jsonld)
- [JSON Schema](https://stencila.org/CodeLocation.schema.json)
- Python class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`CodeLocation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_location.rs)
- TypeScript class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeLocation.ts)

***

This documentation was generated from [`CodeLocation.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeLocation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
