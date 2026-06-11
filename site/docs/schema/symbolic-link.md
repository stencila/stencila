---
title: Symbolic Link
description: A symbolic link on a file system.
---

This is a lightweight representation of a symbolic link as a directory entry,
distinct from the file or directory that the link may point to.

Keeping symbolic links explicit prevents graph and workspace tooling from
silently dereferencing paths and reading content outside the workspace under a
local-looking path. The `path` property identifies the link entry itself, while
`target` preserves the target spelling stored by the filesystem. That
target may be relative, absolute, missing, or outside the workspace; callers
that need to resolve it should do so deliberately and record any relationship
separately.


# Properties

The `SymbolicLink` type has these properties:

| Name     | Description                                                              | Type                    | Inherited from          |
| -------- | ------------------------------------------------------------------------ | ----------------------- | ----------------------- |
| `name`   | The name of the symbolic link.                                           | [`String`](./string.md) | -                       |
| `path`   | The path (absolute or relative) of the symbolic link on the file system. | [`String`](./string.md) | -                       |
| `target` | The raw target path stored by the symbolic link.                         | [`String`](./string.md) | -                       |
| `id`     | The identifier for this item.                                            | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `SymbolicLink` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `SymbolicLink` type is represented in:

- [JSON-LD](https://stencila.org/SymbolicLink.jsonld)
- [JSON Schema](https://stencila.org/SymbolicLink.schema.json)
- Python class [`SymbolicLink`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`SymbolicLink`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/symbolic_link.rs)
- TypeScript class [`SymbolicLink`](https://github.com/stencila/stencila/blob/main/ts/src/types/SymbolicLink.ts)

***

This documentation was generated from [`SymbolicLink.yaml`](https://github.com/stencila/stencila/blob/main/schema/SymbolicLink.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
