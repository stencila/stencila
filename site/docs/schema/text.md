---
title: Text
description: Textual content.
---

Intended mostly for use for inline text e.g. the text in a paragraph.

Differs from the primitive `String` type in that it has a `type` and `id` property.
The `id` property allows use to identify text nodes with a sequence of inline nodes
for better diffing.

Also, in Rust, the `value` property is implemented as a CRDT.


# Properties

The `Text` type has these properties:

| Name                  | Description                                  | Type                                              | Inherited from          |
| --------------------- | -------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `id`                  | The identifier for this item.                | [`String`](./string.md)                           | [`Entity`](./entity.md) |
| `value`               | The value of the text content                | [`Cord`](./cord.md)                               | -                       |
| `compilationMessages` | Messages generated while compiling the text. | [`CompilationMessage`](./compilation-message.md)* | -                       |

# Related

The `Text` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Text` type is represented in:

- [JSON-LD](https://stencila.org/Text.jsonld)
- [JSON Schema](https://stencila.org/Text.schema.json)
- Python class [`Text`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/text.py)
- Rust struct [`Text`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/text.rs)
- TypeScript class [`Text`](https://github.com/stencila/stencila/blob/main/ts/src/types/Text.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Text` type are generated using the following strategies.

::: table

| Property | Complexity | Description                                                                                                                    | Strategy                                                        |
| -------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------- |
| `value`  | Min+       | Generate a fixed string of text.                                                                                               | `Cord::from("text")`                                            |
|          | Low+       | Generate a random string of up to 10 alphanumeric characters.                                                                  | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`                     |
|          | High+      | Generate a random string of up to 100 alphanumeric characters, some special characters commonly used in prose, and whitespace. | `r"[a-zA-Z0-9 \t\-_.!?*+-/()'<>=]{1,100}".prop_map(Cord::from)` |
|          | Max        | Generate an arbitrary string.                                                                                                  | `String::arbitrary().prop_map(Cord::from)`                      |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`Text.yaml`](https://github.com/stencila/stencila/blob/main/schema/Text.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
