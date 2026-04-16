---
title: Compilation Digest
description: A digest of an executable node and its dependencies.
---

This is a type used in Stencila Schema for recording a digest of executable content
and its dependencies.

It exists to support incremental compilation and cache invalidation across
executable document nodes. Rather than reprocessing content unconditionally,
Stencila can compare digests to determine whether derived outputs or compiled
artifacts are still current.

This type is commonly used by properties such as `compilationDigest` and
related compilation metadata on [`Executable`](./executable.md) and derived
types.


# Properties

The `CompilationDigest` type has these properties:

| Name                 | Description                                                                 | Type                                       | Inherited from          |
| -------------------- | --------------------------------------------------------------------------- | ------------------------------------------ | ----------------------- |
| `stateDigest`        | A digest of the state of a node.                                            | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `semanticDigest`     | A digest of the semantics of the node with respect to the dependency graph. | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `dependenciesDigest` | A digest of the semantic digests of the dependencies of a node.             | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `dependenciesStale`  | A count of the number of dependencies that are stale.                       | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `dependenciesFailed` | A count of the number of dependencies that failed.                          | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `id`                 | The identifier for this item.                                               | [`String`](./string.md)                    | [`Entity`](./entity.md) |

# Related

The `CompilationDigest` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `CompilationDigest` type is represented in:

- [JSON-LD](https://stencila.org/CompilationDigest.jsonld)
- [JSON Schema](https://stencila.org/CompilationDigest.schema.json)
- Python class [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/compilation_digest.rs)
- TypeScript class [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/ts/src/types/CompilationDigest.ts)

***

This documentation was generated from [`CompilationDigest.yaml`](https://github.com/stencila/stencila/blob/main/schema/CompilationDigest.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
