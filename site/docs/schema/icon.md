---
title: Icon
description: An icon, typically rendered using an icon font.
---

This is a type used in Stencila Schema for icons embedded in document content.

It exists so icons can be represented structurally and styled consistently
across renderers rather than being reduced to opaque HTML or font markup. This
supports document-authoring and publishing workflows that need portable icon
references.

Key properties identify the icon set, icon name, and any presentational hints
needed by renderers.


# Analogues

The following external types, elements, or nodes are similar to a `Icon`:

- [HTML icon element pattern](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/i): Approximate HTML analogue because icons are often represented using `<i>` or `<span>` with classes, but HTML has no dedicated semantic icon element.

# Properties

The `Icon` type has these properties:

| Name         | Description                                          | Type                      | Inherited from          |
| ------------ | ---------------------------------------------------- | ------------------------- | ----------------------- |
| `name`       | The name of the icon e.g. "clock" or "lucide:clock". | [`String`](./string.md)   | -                       |
| `label`      | An accessible text label for the icon.               | [`String`](./string.md)   | -                       |
| `decorative` | Whether the icon is purely decorative.               | [`Boolean`](./boolean.md) | -                       |
| `style`      | Tailwind utility classes to apply to the icon.       | [`String`](./string.md)   | -                       |
| `id`         | The identifier for this item.                        | [`String`](./string.md)   | [`Entity`](./entity.md) |

# Related

The `Icon` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Icon` type is represented in:

- [JSON-LD](https://stencila.org/Icon.jsonld)
- [JSON Schema](https://stencila.org/Icon.schema.json)
- Python class [`Icon`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Icon`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/icon.rs)
- TypeScript class [`Icon`](https://github.com/stencila/stencila/blob/main/ts/src/types/Icon.ts)

***

This documentation was generated from [`Icon.yaml`](https://github.com/stencila/stencila/blob/main/schema/Icon.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
