---
title: Directory
description: A directory on the file system.
---

Previously this type extended `Collection` (which in turn extends `CreativeWork`).
However, to avoid consuming more memory that necessary when creating directory listings
with many directories, it now extends `Entity`.


# Properties

The `Directory` type has these properties:

| Name    | Description                                                     | Type                                                    | Inherited from          |
| ------- | --------------------------------------------------------------- | ------------------------------------------------------- | ----------------------- |
| `id`    | The identifier for this item.                                   | [`String`](./string.md)                                 | [`Entity`](./entity.md) |
| `name`  | The name of the directory.                                      | [`String`](./string.md)                                 | -                       |
| `path`  | The path (absolute or relative) of the file on the file system. | [`String`](./string.md)                                 | -                       |
| `parts` | The files and other directories within this directory.          | ([`File`](./file.md) \| [`Directory`](./directory.md))* | -                       |

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
