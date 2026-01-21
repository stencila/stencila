---
title: Math Block
description: A block of math, e.g an equation, to be treated as block content.
---

# Properties

The `MathBlock` type has these properties:

| Name                  | Description                                                         | Type                                              | Inherited from          |
| --------------------- | ------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `id`                  | The identifier for this item.                                       | [`String`](./string.md)                           | [`Entity`](./entity.md) |
| `code`                | The code of the equation in the `mathLanguage`.                     | [`Cord`](./cord.md)                               | [`Math`](./math.md)     |
| `mathLanguage`        | The language used for the equation e.g tex, mathml, asciimath.      | [`String`](./string.md)                           | [`Math`](./math.md)     |
| `authors`             | The authors of the math.                                            | [`Author`](./author.md)*                          | [`Math`](./math.md)     |
| `provenance`          | A summary of the provenance of the math.                            | [`ProvenanceCount`](./provenance-count.md)*       | [`Math`](./math.md)     |
| `compilationDigest`   | A digest of the `code` and `mathLanguage`.                          | [`CompilationDigest`](./compilation-digest.md)    | [`Math`](./math.md)     |
| `compilationMessages` | Messages generated while parsing and compiling the math expression. | [`CompilationMessage`](./compilation-message.md)* | [`Math`](./math.md)     |
| `mathml`              | The MathML transpiled from the `code`.                              | [`String`](./string.md)                           | [`Math`](./math.md)     |
| `images`              | Images of the math.                                                 | [`ImageObject`](./image-object.md)*               | [`Math`](./math.md)     |
| `label`               | A short label for the math block.                                   | [`String`](./string.md)                           | -                       |
| `labelAutomatically`  | Whether the label should be automatically updated.                  | [`Boolean`](./boolean.md)                         | -                       |

# Related

The `MathBlock` type is related to these types:

- Parents: [`Math`](./math.md)
- Children: none

# Bindings

The `MathBlock` type is represented in:

- [JSON-LD](https://stencila.org/MathBlock.jsonld)
- [JSON Schema](https://stencila.org/MathBlock.schema.json)
- Python class [`MathBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/math_block.py)
- Rust struct [`MathBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math_block.rs)
- TypeScript class [`MathBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/MathBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `MathBlock` type are generated using the following strategies.

::: table

| Property       | Complexity | Description                                                                                                                                          | Strategy                                    |
| -------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `code`         | Min+       | Generate a simple fixed string of math.                                                                                                              | `Cord::from("math")`                        |
|                | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which <br><br>when leading or trailing causes issues for Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)` |
|                | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                                     | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`   |
|                | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary().prop_map(Cord::from)`  |
| `mathLanguage` | Min+       | Fixed as TeX (for testing with Markdown which uses dollars to delimit TeX by default)                                                                | `Some(String::from("tex"))`                 |
|                | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                                        | `option::of(r"[a-zA-Z0-9]{1,10}")`          |
|                | Max        | Generate an arbitrary string.                                                                                                                        | `option::of(String::arbitrary())`           |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`MathBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/MathBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
