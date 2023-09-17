---
title:
- type: Text
  value: Strong
---

# Strong

**Strongly emphasized content.**

**`@id`**: `stencila:Strong`

## Properties

The `Strong` type has these properties:

| Name    | `@id`                                | Type                                                                 | Description                  | Inherited from                                                      |
| ------- | ------------------------------------ | -------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| content | `stencila:content`                   | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)* | The content that is marked.  | [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)     |

## Related

The `Strong` type is related to these types:

- Parents: [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)
- Children: none

## Formats

The `Strong` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                  |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | -------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag `<strong>`              |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟢 No loss      |              | 🚧 Under development    | Encoded using template `**{content}**` |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                        |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                        |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                        |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                        |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                        |

## Bindings

The `Strong` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Strong.jsonld)
- [JSON Schema](https://stencila.dev/Strong.schema.json)
- Python class [`Strong`](https://github.com/stencila/stencila/blob/main/python/stencila/types/strong.py)
- Rust struct [`Strong`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/strong.rs)
- TypeScript class [`Strong`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Strong.ts)

## Source

This documentation was generated from [`Strong.yaml`](https://github.com/stencila/stencila/blob/main/schema/Strong.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).