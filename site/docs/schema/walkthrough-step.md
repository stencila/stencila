---
title: Walkthrough Step
description: A step in a walkthrough.
---

This type is marked as unstable and is subject to change.

# Properties

The `WalkthroughStep` type has these properties:

| Name          | Description                                                                      | Type                      | Inherited from          |
| ------------- | -------------------------------------------------------------------------------- | ------------------------- | ----------------------- |
| `id`          | The identifier for this item.                                                    | [`String`](./string.md)   | [`Entity`](./entity.md) |
| `isCollapsed` | Whether this step is active (i.e. is encoded in source format and can be edited) | [`Boolean`](./boolean.md) | -                       |
| `content`     | The content of the step.                                                         | [`Block`](./block.md)*    | -                       |

# Related

The `WalkthroughStep` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `WalkthroughStep` type is represented in:

- [JSON-LD](https://stencila.org/WalkthroughStep.jsonld)
- [JSON Schema](https://stencila.org/WalkthroughStep.schema.json)
- Python class [`WalkthroughStep`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/walkthrough_step.py)
- Rust struct [`WalkthroughStep`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/walkthrough_step.rs)
- TypeScript class [`WalkthroughStep`](https://github.com/stencila/stencila/blob/main/ts/src/types/WalkthroughStep.ts)

# Source

This documentation was generated from [`WalkthroughStep.yaml`](https://github.com/stencila/stencila/blob/main/schema/WalkthroughStep.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
