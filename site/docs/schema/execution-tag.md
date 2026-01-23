---
title: Execution Tag
description: A tag on code that affects its execution.
---

# Properties

The `ExecutionTag` type has these properties:

| Name       | Description                               | Type                      | Inherited from          |
| ---------- | ----------------------------------------- | ------------------------- | ----------------------- |
| `id`       | The identifier for this item.             | [`String`](./string.md)   | [`Entity`](./entity.md) |
| `name`     | The name of the tag                       | [`String`](./string.md)   | -                       |
| `value`    | The value of the tag                      | [`String`](./string.md)   | -                       |
| `isGlobal` | Whether the tag is global to the document | [`Boolean`](./boolean.md) | -                       |

# Related

The `ExecutionTag` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ExecutionTag` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionTag.jsonld)
- [JSON Schema](https://stencila.org/ExecutionTag.schema.json)
- Python class [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_tag.rs)
- TypeScript class [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionTag.ts)

***

This documentation was generated from [`ExecutionTag.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionTag.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
