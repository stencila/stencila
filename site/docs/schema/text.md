---
title: Text
description: A text node.
---

This is an implementation of schema.org [`Text`](https://schema.org/Text),
adapted in Stencila Schema as a node type rather than a primitive string.

It exists so textual content can participate in the document tree with its own
identity and metadata, which is useful for diffing, editing, and collaborative
workflows. This differs from the primitive [`String`](./string.md) type, which
represents plain scalar string values outside the node model.

The main property is `value`.


# Analogues

The following external types, elements, or nodes are similar to a `Text`:

- schema.org [`Text`](https://schema.org/Text): Direct schema.org source type, adapted in Stencila from a scalar data type into a first-class node.
- [HTML text node](https://dom.spec.whatwg.org/#text): Closest DOM analogue for literal text content, though Stencila text nodes can carry identity and metadata through `Entity`.
- Pandoc [`Str`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Str): Close Pandoc analogue for plain inline text, though Pandoc splits spaces and line breaks into separate inline constructors.
- MDAST [`Text`](https://github.com/syntax-tree/mdast#text): Closest MDAST analogue for text leaf nodes.

# Properties

The `Text` type has these properties:

| Name                  | Description                                  | Type                                              | Inherited from          |
| --------------------- | -------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `value`               | The value of the text content                | [`Cord`](./cord.md)                               | -                       |
| `compilationMessages` | Messages generated while compiling the text. | [`CompilationMessage`](./compilation-message.md)* | -                       |
| `id`                  | The identifier for this item.                | [`String`](./string.md)                           | [`Entity`](./entity.md) |

# Related

The `Text` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Text` type is represented in:

- [JSON-LD](https://stencila.org/Text.jsonld)
- [JSON Schema](https://stencila.org/Text.schema.json)
- Python class [`Text`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Text.yaml`](https://github.com/stencila/stencila/blob/main/schema/Text.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
