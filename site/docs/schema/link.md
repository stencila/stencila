---
title: Link
description: A hyperlink to other pages, sections within the same document, resources, or any URL.
---

# Properties

The `Link` type has these properties:

| Name                  | Description                                                                                                         | Type                                              | Inherited from          |
| --------------------- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `id`                  | The identifier for this item.                                                                                       | [`String`](./string.md)                           | [`Entity`](./entity.md) |
| `content`             | The textual content of the link.                                                                                    | [`Inline`](./inline.md)*                          | -                       |
| `target`              | The target of the link.                                                                                             | [`String`](./string.md)                           | -                       |
| `title`               | A title for the link.                                                                                               | [`String`](./string.md)                           | -                       |
| `rel`                 | The relation between the target and the current thing.                                                              | [`String`](./string.md)                           | -                       |
| `labelOnly`           | Only show the label of the internal target (e.g. "2"), rather than both the label type and label (e.g. "Figure 2"). | [`Boolean`](./boolean.md)                         | -                       |
| `compilationMessages` | Messages generated while compiling the link (e.g. missing internal link or invalid external link).                  | [`CompilationMessage`](./compilation-message.md)* | -                       |

# Related

The `Link` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Link` type is represented in:

- [JSON-LD](https://stencila.org/Link.jsonld)
- [JSON Schema](https://stencila.org/Link.schema.json)
- Python class [`Link`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/link.py)
- Rust struct [`Link`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/link.rs)
- TypeScript class [`Link`](https://github.com/stencila/stencila/blob/main/ts/src/types/Link.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Link` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`Link.yaml`](https://github.com/stencila/stencila/blob/main/schema/Link.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
