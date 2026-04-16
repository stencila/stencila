---
title: Math Inline
description: A fragment of math, e.g a variable name, to be treated as inline content.
---

This is an inline representation used in Stencila Schema for mathematical
content.

It extends [`Math`](./math.md) for inline equations, symbols, and variables
that occur within prose. This keeps inline math aligned with the broader math
model while distinguishing it from display math in rendering and authoring
workflows.

Key properties are inherited from [`Math`](./math.md).


# Analogues

The following external types, elements, or nodes are similar to a `MathInline`:

- JATS [`<inline-formula>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/inline-formula.html)
- Pandoc [`Math`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Math): Close Pandoc analogue for inline math, though Pandoc distinguishes inline mode via a subkind rather than a separate node type.
- MDAST [`InlineMath`](https://github.com/syntax-tree/mdast#inlinemath)

# Properties

The `MathInline` type has these properties:

| Name                  | Description                                                         | Type                                              | Inherited from          |
| --------------------- | ------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `code`                | The code of the equation in the `mathLanguage`.                     | [`Cord`](./cord.md)                               | [`Math`](./math.md)     |
| `mathLanguage`        | The language used for the equation e.g tex, mathml, asciimath.      | [`String`](./string.md)                           | [`Math`](./math.md)     |
| `authors`             | The authors of the math.                                            | [`Author`](./author.md)*                          | [`Math`](./math.md)     |
| `provenance`          | A summary of the provenance of the math.                            | [`ProvenanceCount`](./provenance-count.md)*       | [`Math`](./math.md)     |
| `compilationDigest`   | A digest of the `code` and `mathLanguage`.                          | [`CompilationDigest`](./compilation-digest.md)    | [`Math`](./math.md)     |
| `compilationMessages` | Messages generated while parsing and compiling the math expression. | [`CompilationMessage`](./compilation-message.md)* | [`Math`](./math.md)     |
| `mathml`              | The MathML transpiled from the `code`.                              | [`String`](./string.md)                           | [`Math`](./math.md)     |
| `images`              | Images of the math.                                                 | [`ImageObject`](./image-object.md)*               | [`Math`](./math.md)     |
| `id`                  | The identifier for this item.                                       | [`String`](./string.md)                           | [`Entity`](./entity.md) |

# Related

The `MathInline` type is related to these types:

- Parents: [`Math`](./math.md)
- Children: none

# Bindings

The `MathInline` type is represented in:

- [JSON-LD](https://stencila.org/MathInline.jsonld)
- [JSON Schema](https://stencila.org/MathInline.schema.json)
- Python class [`MathInline`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`MathInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math_inline.rs)
- TypeScript class [`MathInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/MathInline.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `MathInline` type are generated using the following strategies.

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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`MathInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/MathInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
