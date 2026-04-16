---
title: Admonition Type
description: A category of admonition.
---

This is an enumeration used in Stencila Schema for classifying
[`Admonition`](./admonition.md) nodes.

It exists to give admonitions a small controlled vocabulary that can be mapped
consistently to authoring, styling, and publishing formats. The enumeration
captures the semantic intent of the callout rather than its visual
presentation.

See [`Admonition.type`](./admonition.md#type) for the property that uses this
enumeration.


# Members

The `AdmonitionType` type has these members:

| Member      | Description |
| ----------- | ----------- |
| `Note`      | -           |
| `Info`      | -           |
| `Tip`       | -           |
| `Important` | -           |
| `Success`   | -           |
| `Failure`   | -           |
| `Warning`   | -           |
| `Danger`    | -           |
| `Error`     | -           |

# Bindings

The `AdmonitionType` type is represented in:

- [JSON-LD](https://stencila.org/AdmonitionType.jsonld)
- [JSON Schema](https://stencila.org/AdmonitionType.schema.json)
- Python type [`AdmonitionType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`AdmonitionType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/admonition_type.rs)
- TypeScript type [`AdmonitionType`](https://github.com/stencila/stencila/blob/main/ts/src/types/AdmonitionType.ts)

***

This documentation was generated from [`AdmonitionType.yaml`](https://github.com/stencila/stencila/blob/main/schema/AdmonitionType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
