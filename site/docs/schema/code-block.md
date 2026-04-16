---
title: Code Block
description: A code block.
---

This is a block representation used in Stencila Schema for non-executable
code.

It extends [`CodeStatic`](./code-static.md) for code that should be displayed
and preserved as document content without normal execution semantics. Stencila
also adds optional demo compilation behavior so some static code blocks can
render derived content while remaining distinct from executable code chunks.

Key properties are inherited from [`CodeStatic`](./code-static.md), with
Stencila Schema behavior centered on `isDemo`, `content`, and the associated
compilation properties.


# Analogues

The following external types, elements, or nodes are similar to a `CodeBlock`:

- HTML [`<pre>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre)
- JATS [`<code>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/code.html)
- Pandoc [`CodeBlock`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:CodeBlock)
- MDAST [`Code`](https://github.com/syntax-tree/mdast#code): Closest MDAST analogue for fenced or indented code blocks.

# Properties

The `CodeBlock` type has these properties:

| Name                  | Description                                                    | Type                                              | Inherited from                   |
| --------------------- | -------------------------------------------------------------- | ------------------------------------------------- | -------------------------------- |
| `isDemo`              | Whether the code block is a demo that should also be rendered. | [`Boolean`](./boolean.md)                         | -                                |
| `compilationDigest`   | A digest of the `code` and `programmingLanguage`.              | [`CompilationDigest`](./compilation-digest.md)    | -                                |
| `compilationMessages` | Messages generated while compiling the demo content.           | [`CompilationMessage`](./compilation-message.md)* | -                                |
| `content`             | The content rendered from the code when `isDemo` is true.      | [`Block`](./block.md)*                            | -                                |
| `code`                | The code.                                                      | [`Cord`](./cord.md)                               | [`CodeStatic`](./code-static.md) |
| `programmingLanguage` | The programming language of the code.                          | [`String`](./string.md)                           | [`CodeStatic`](./code-static.md) |
| `authors`             | The authors of the code.                                       | [`Author`](./author.md)*                          | [`CodeStatic`](./code-static.md) |
| `provenance`          | A summary of the provenance of the code.                       | [`ProvenanceCount`](./provenance-count.md)*       | [`CodeStatic`](./code-static.md) |
| `id`                  | The identifier for this item.                                  | [`String`](./string.md)                           | [`Entity`](./entity.md)          |

# Related

The `CodeBlock` type is related to these types:

- Parents: [`CodeStatic`](./code-static.md)
- Children: none

# Bindings

The `CodeBlock` type is represented in:

- [JSON-LD](https://stencila.org/CodeBlock.jsonld)
- [JSON Schema](https://stencila.org/CodeBlock.schema.json)
- Python class [`CodeBlock`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`CodeBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_block.rs)
- TypeScript class [`CodeBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `CodeBlock` type are generated using the following strategies.

::: table

| Property              | Complexity | Description                                                                                                                    | Strategy                                      |
| --------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------- |
| `code`                | Min+       | Generate a simple fixed string of code.                                                                                        | `Cord::from("code")`                          |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which<br><br>can be problematic in Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`   |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                               | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`     |
|                       | Max        | Generate an arbitrary string.                                                                                                  | `String::arbitrary().prop_map(Cord::from)`    |
| `programmingLanguage` | Min+       | Do not generate a programming language.                                                                                        | `None`                                        |
|                       | Low+       | Generate one of the well known programming language short names.                                                               | `option::of(r"(cpp)\|(js)\|(py)\|(r)\|(ts)")` |
|                       | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                  | `option::of(r"[a-zA-Z0-9]{1,10}")`            |
|                       | Max        | Generate an arbitrary string.                                                                                                  | `option::of(String::arbitrary())`             |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`CodeBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
