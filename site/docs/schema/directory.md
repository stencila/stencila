---
title: Directory
description: A directory on a file system.
---

This is a type used in Stencila Schema for directories in a file system.

It exists to represent directories as lightweight filesystem entities that can
be inspected, listed, and linked from workflows and documents. Unlike more
metadata-rich work types, it extends [`Entity`](./entity.md) rather than
[`Collection`](./collection.md) to avoid carrying unnecessary creative-work
overhead when working with large directory listings.

Key properties focus on filesystem identity and structure rather than
publishable content.


# Properties

The `Directory` type has these properties:

| Name    | Description                                                     | Type                                                    | Inherited from          |
| ------- | --------------------------------------------------------------- | ------------------------------------------------------- | ----------------------- |
| `name`  | The name of the directory.                                      | [`String`](./string.md)                                 | -                       |
| `path`  | The path (absolute or relative) of the file on the file system. | [`String`](./string.md)                                 | -                       |
| `parts` | The files and other directories within this directory.          | ([`File`](./file.md) \| [`Directory`](./directory.md))* | -                       |
| `id`    | The identifier for this item.                                   | [`String`](./string.md)                                 | [`Entity`](./entity.md) |

# Related

The `Directory` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Directory` type is represented in:

- [JSON-LD](https://stencila.org/Directory.jsonld)
- [JSON Schema](https://stencila.org/Directory.schema.json)
- Python class [`Directory`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Directory`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/directory.rs)
- TypeScript class [`Directory`](https://github.com/stencila/stencila/blob/main/ts/src/types/Directory.ts)

***

This documentation was generated from [`Directory.yaml`](https://github.com/stencila/stencila/blob/main/schema/Directory.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
