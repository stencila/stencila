---
title: Walkthrough
description: An interactive walkthrough made up of several, successively revealed steps.
---

This type is marked as unstable and is subject to change.

# Properties

The `Walkthrough` type has these properties:

| Name          | Description                          | Type                                        | Inherited from          |
| ------------- | ------------------------------------ | ------------------------------------------- | ----------------------- |
| `id`          | The identifier for this item.        | [`String`](./string.md)                     | [`Entity`](./entity.md) |
| `isCollapsed` | Whether the walkthrough is collapsed | [`Boolean`](./boolean.md)                   | -                       |
| `steps`       | The steps making up the walkthrough. | [`WalkthroughStep`](./walkthrough-step.md)* | -                       |

# Related

The `Walkthrough` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Walkthrough` type is represented in:

- [JSON-LD](https://stencila.org/Walkthrough.jsonld)
- [JSON Schema](https://stencila.org/Walkthrough.schema.json)
- Python class [`Walkthrough`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/walkthrough.py)
- Rust struct [`Walkthrough`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/walkthrough.rs)
- TypeScript class [`Walkthrough`](https://github.com/stencila/stencila/blob/main/ts/src/types/Walkthrough.ts)

# Source

This documentation was generated from [`Walkthrough.yaml`](https://github.com/stencila/stencila/blob/main/schema/Walkthrough.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
