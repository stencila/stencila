---
title: Icon
description: An icon, typically rendered using an icon font.
---

# Properties

The `Icon` type has these properties:

| Name         | Description                                          | Type                      | Inherited from          |
| ------------ | ---------------------------------------------------- | ------------------------- | ----------------------- |
| `id`         | The identifier for this item.                        | [`String`](./string.md)   | [`Entity`](./entity.md) |
| `name`       | The name of the icon e.g. "clock" or "lucide:clock". | [`String`](./string.md)   | -                       |
| `label`      | An accessible text label for the icon.               | [`String`](./string.md)   | -                       |
| `decorative` | Whether the icon is purely decorative.               | [`Boolean`](./boolean.md) | -                       |
| `style`      | Tailwind utility classes to apply to the icon.       | [`String`](./string.md)   | -                       |

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
