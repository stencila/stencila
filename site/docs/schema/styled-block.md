---
title: Styled Block
description: Styled block content.
---

This type is marked as unstable and is subject to change.

# Properties

The `StyledBlock` type has these properties:

| Name                  | Description                                                            | Type                                              | Inherited from          |
| --------------------- | ---------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `id`                  | The identifier for this item.                                          | [`String`](./string.md)                           | [`Entity`](./entity.md) |
| `code`                | The code of the equation in the `styleLanguage`.                       | [`Cord`](./cord.md)                               | [`Styled`](./styled.md) |
| `styleLanguage`       | The language used for the style specification e.g. css, tw             | [`String`](./string.md)                           | [`Styled`](./styled.md) |
| `authors`             | The authors of the code and content in the styled node.                | [`Author`](./author.md)*                          | [`Styled`](./styled.md) |
| `provenance`          | A summary of the provenance of the code and content in the styed node. | [`ProvenanceCount`](./provenance-count.md)*       | [`Styled`](./styled.md) |
| `compilationDigest`   | A digest of the `code` and `styleLanguage`.                            | [`CompilationDigest`](./compilation-digest.md)    | [`Styled`](./styled.md) |
| `compilationMessages` | Messages generated while parsing and transpiling the style.            | [`CompilationMessage`](./compilation-message.md)* | [`Styled`](./styled.md) |
| `css`                 | A Cascading Style Sheet (CSS) transpiled from the `code` property.     | [`String`](./string.md)                           | [`Styled`](./styled.md) |
| `classList`           | A space separated list of class names associated with the node.        | [`String`](./string.md)                           | [`Styled`](./styled.md) |
| `content`             | The content within the styled block                                    | [`Block`](./block.md)*                            | -                       |

# Related

The `StyledBlock` type is related to these types:

- Parents: [`Styled`](./styled.md)
- Children: [`Page`](./page.md)

# Bindings

The `StyledBlock` type is represented in:

- [JSON-LD](https://stencila.org/StyledBlock.jsonld)
- [JSON Schema](https://stencila.org/StyledBlock.schema.json)
- Python class [`StyledBlock`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`StyledBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/styled_block.rs)
- TypeScript class [`StyledBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/StyledBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `StyledBlock` type are generated using the following strategies.

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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`StyledBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/StyledBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
