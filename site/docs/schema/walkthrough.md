---
title: Walkthrough
description: An interactive walkthrough.
---

This is a type used in Stencila Schema for interactive walkthroughs embedded in
documents.

It exists to support guided, stepwise experiences for readers and authors,
with each step represented as structured document content rather than external
application logic. This makes walkthroughs portable across Stencila tools and
publishable outputs.

Key properties include the ordered `steps` and any metadata controlling
progression or presentation.


This type is marked as unstable and is subject to change.

# Properties

The `Walkthrough` type has these properties:

| Name          | Description                          | Type                                        | Inherited from          |
| ------------- | ------------------------------------ | ------------------------------------------- | ----------------------- |
| `isCollapsed` | Whether the walkthrough is collapsed | [`Boolean`](./boolean.md)                   | -                       |
| `steps`       | The steps making up the walkthrough. | [`WalkthroughStep`](./walkthrough-step.md)* | -                       |
| `id`          | The identifier for this item.        | [`String`](./string.md)                     | [`Entity`](./entity.md) |

# Related

The `Walkthrough` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Walkthrough` type is represented in:

- [JSON-LD](https://stencila.org/Walkthrough.jsonld)
- [JSON Schema](https://stencila.org/Walkthrough.schema.json)
- Python class [`Walkthrough`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Walkthrough`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/walkthrough.rs)
- TypeScript class [`Walkthrough`](https://github.com/stencila/stencila/blob/main/ts/src/types/Walkthrough.ts)

***

This documentation was generated from [`Walkthrough.yaml`](https://github.com/stencila/stencila/blob/main/schema/Walkthrough.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
