---
title: Page
description: A separate page in a document
---

This type is marked as unstable and is subject to change.

# Properties

The `Page` type has these properties:

| Name                  | Description                                                            | Type                                              | Inherited from                     |
| --------------------- | ---------------------------------------------------------------------- | ------------------------------------------------- | ---------------------------------- |
| `id`                  | The identifier for this item.                                          | [`String`](./string.md)                           | [`Entity`](./entity.md)            |
| `code`                | The code of the equation in the `styleLanguage`.                       | [`Cord`](./cord.md)                               | [`Styled`](./styled.md)            |
| `styleLanguage`       | The language used for the style specification e.g. css, tw             | [`String`](./string.md)                           | [`Styled`](./styled.md)            |
| `authors`             | The authors of the code and content in the styled node.                | [`Author`](./author.md)*                          | [`Styled`](./styled.md)            |
| `provenance`          | A summary of the provenance of the code and content in the styed node. | [`ProvenanceCount`](./provenance-count.md)*       | [`Styled`](./styled.md)            |
| `compilationDigest`   | A digest of the `code` and `styleLanguage`.                            | [`CompilationDigest`](./compilation-digest.md)    | [`Styled`](./styled.md)            |
| `compilationMessages` | Messages generated while parsing and transpiling the style.            | [`CompilationMessage`](./compilation-message.md)* | [`Styled`](./styled.md)            |
| `css`                 | A Cascading Style Sheet (CSS) transpiled from the `code` property.     | [`String`](./string.md)                           | [`Styled`](./styled.md)            |
| `classList`           | A space separated list of class names associated with the node.        | [`String`](./string.md)                           | [`Styled`](./styled.md)            |
| `content`             | The content within the styled block                                    | [`Block`](./block.md)*                            | [`StyledBlock`](./styled-block.md) |

# Related

The `Page` type is related to these types:

- Parents: [`StyledBlock`](./styled-block.md)
- Children: none

# Bindings

The `Page` type is represented in:

- [JSON-LD](https://stencila.org/Page.jsonld)
- [JSON Schema](https://stencila.org/Page.schema.json)
- Python class [`Page`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/page.py)
- Rust struct [`Page`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/page.rs)
- TypeScript class [`Page`](https://github.com/stencila/stencila/blob/main/ts/src/types/Page.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Page` type are generated using the following strategies.

::: table

| Property        | Complexity | Description                                                                                                                       | Strategy                                                                                                                                                                    |
| --------------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `code`          | Min+       | Generate a simple fixed string of code.                                                                                           | `Cord::from("code")`                                                                                                                                                        |
|                 | Low+       | Generate a random string of up to 10 alphanumeric & space characters (trimmed). Avoid keywords used to identify other node types. | `r"[a-zA-Z0-9 ]{1,10}".prop_filter("No keywords", \|code\| !["include", "call", "if", "ifblock", "for"].contains(&code.trim())).prop_map(\|code\| Cord::from(code.trim()))` |
|                 | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                  | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`                                                                                                                                   |
|                 | Max        | Generate an arbitrary string.                                                                                                     | `String::arbitrary().prop_map(Cord::from)`                                                                                                                                  |
| `styleLanguage` | Min+       | Do not generate a style language.                                                                                                 | `None`                                                                                                                                                                      |
|                 | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                     | `option::of(r"[a-zA-Z0-9]{1,10}")`                                                                                                                                          |
|                 | Max        | Generate an arbitrary string.                                                                                                     | `option::of(String::arbitrary())`                                                                                                                                           |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`Page.yaml`](https://github.com/stencila/stencila/blob/main/schema/Page.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
