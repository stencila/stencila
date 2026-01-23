---
title: Cord
description: A value comprised of a sequence of characters.
---

This type exists to differentiate between between a plain string of characters
(which is modified by complete replacement) and a sequence of characters stored and
synchronized as a CRDT (which is is modified by insertions and deletions).

Its use includes the `value` property of the `Text` type and the `code`
property of `CodeExecutable` nodes.


# Bindings

The `Cord` type is represented in:

- [JSON-LD](https://stencila.org/Cord.jsonld)
- [JSON Schema](https://stencila.org/Cord.schema.json)
- Python type [`Cord`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Cord`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/cord.rs)
- TypeScript type [`Cord`](https://github.com/stencila/stencila/blob/main/ts/src/types/Cord.ts)

***

This documentation was generated from [`Cord.yaml`](https://github.com/stencila/stencila/blob/main/schema/Cord.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
