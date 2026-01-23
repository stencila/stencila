---
title: Appendix Break
description: A break in a document indicating the start one or more appendices.
---

Similar to a LaTeX `\appendix` command, this node causes level one headings to
have an appendix label and figure and table numbering to switch to be prefixed
by 'A' (for the first appendix), 'B', and so on. A document should only have
one `AppendixBreak`.


# Properties

The `AppendixBreak` type has these properties:

| Name                  | Description                                            | Type                                              | Inherited from          |
| --------------------- | ------------------------------------------------------ | ------------------------------------------------- | ----------------------- |
| `id`                  | The identifier for this item.                          | [`String`](./string.md)                           | [`Entity`](./entity.md) |
| `compilationMessages` | Messages generated while compiling the appendix break. | [`CompilationMessage`](./compilation-message.md)* | -                       |

# Related

The `AppendixBreak` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `AppendixBreak` type is represented in:

- [JSON-LD](https://stencila.org/AppendixBreak.jsonld)
- [JSON Schema](https://stencila.org/AppendixBreak.schema.json)
- Python class [`AppendixBreak`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`AppendixBreak`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/appendix_break.rs)
- TypeScript class [`AppendixBreak`](https://github.com/stencila/stencila/blob/main/ts/src/types/AppendixBreak.ts)

***

This documentation was generated from [`AppendixBreak.yaml`](https://github.com/stencila/stencila/blob/main/schema/AppendixBreak.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
