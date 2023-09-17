---
title:
- type: Text
  value: String
---

# String

**A value comprised of a string of characters**

**`@id`**: [`schema:Text`](https://schema.org/Text)

## Formats

The `String` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding      | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | ------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss    |              | 🚧 Under development    |       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss    |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟢 No loss     |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟢 No loss     |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss    |              | 🟢 Stable               |       |

## Bindings

The `String` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/String.jsonld)
- [JSON Schema](https://stencila.dev/String.schema.json)
- Python type [`String`](https://github.com/stencila/stencila/blob/main/python/stencila/types/string.py)
- Rust type [`String`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/string.rs)
- TypeScript type [`String`](https://github.com/stencila/stencila/blob/main/typescript/src/types/String.ts)

## Source

This documentation was generated from [`String.yaml`](https://github.com/stencila/stencila/blob/main/schema/String.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).