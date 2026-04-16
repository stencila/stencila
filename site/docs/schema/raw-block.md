---
title: Raw Block
description: A block of raw content in a specific format.
---

This is a block type used in Stencila Schema for raw format-specific content.

It exists to preserve content that should bypass normal decoding into the
document model and instead be emitted directly when the target format matches.
This is useful for passthrough markup, format-specific extensions, and other
content that Stencila should carry without interpreting structurally.

Key properties include `format`, `content`, and derived compilation-related
properties such as `css` when applicable.


# Properties

The `RawBlock` type has these properties:

| Name                  | Description                                                                             | Type                                              | Inherited from          |
| --------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `format`              | The format of the raw content.                                                          | [`String`](./string.md)                           | -                       |
| `content`             | The raw content.                                                                        | [`Cord`](./cord.md)                               | -                       |
| `compilationDigest`   | A digest of the `format` and `content` properties.                                      | [`CompilationDigest`](./compilation-digest.md)    | -                       |
| `compilationMessages` | Messages generated while parsing and transpiling the `content` into the `css` property. | [`CompilationMessage`](./compilation-message.md)* | -                       |
| `css`                 | A Cascading Style Sheet (CSS) generated from the `content`.                             | [`String`](./string.md)                           | -                       |
| `authors`             | The authors of the content.                                                             | [`Author`](./author.md)*                          | -                       |
| `provenance`          | A summary of the provenance of the content.                                             | [`ProvenanceCount`](./provenance-count.md)*       | -                       |
| `id`                  | The identifier for this item.                                                           | [`String`](./string.md)                           | [`Entity`](./entity.md) |

# Related

The `RawBlock` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `RawBlock` type is represented in:

- [JSON-LD](https://stencila.org/RawBlock.jsonld)
- [JSON Schema](https://stencila.org/RawBlock.schema.json)
- Python class [`RawBlock`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`RawBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/raw_block.rs)
- TypeScript class [`RawBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/RawBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `RawBlock` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                                                                                                                          | Strategy                                    |
| --------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `format`  | Min+       | Fixed as Markdown                                                                                                                                    | `String::from("markdown")`                  |
|           | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                                        | `r"[a-zA-Z0-9]{1,10}"`                      |
|           | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary()`                       |
| `content` | Min+       | Generate a simple fixed string.                                                                                                                      | `Cord::from("content")`                     |
|           | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which <br><br>when leading or trailing causes issues for Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)` |
|           | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                                     | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`   |
|           | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary().prop_map(Cord::from)`  |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`RawBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/RawBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
