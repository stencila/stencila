---
title: Appendix Break
description: A break marking the start of appendices.
---

This is a structural marker used in Stencila Schema for appendix mode in a
document.

It exists to signal that subsequent top-level sections should be treated as
appendices, affecting heading labels and figure or table numbering in a way
similar to the LaTeX `\appendix` command.

This node is mainly relevant to document compilation and labeling workflows. A
document should usually contain at most one `AppendixBreak`.


# Analogues

The following external types, elements, or nodes are similar to a `AppendixBreak`:

- [LaTeX \appendix](https://www.latex-project.org/): Closest functional analogue; like `\appendix`, it switches subsequent section labeling into appendix mode rather than representing visible prose content.

# Properties

The `AppendixBreak` type has these properties:

| Name                  | Description                                            | Type                                              | Inherited from          |
| --------------------- | ------------------------------------------------------ | ------------------------------------------------- | ----------------------- |
| `compilationMessages` | Messages generated while compiling the appendix break. | [`CompilationMessage`](./compilation-message.md)* | -                       |
| `id`                  | The identifier for this item.                          | [`String`](./string.md)                           | [`Entity`](./entity.md) |

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
