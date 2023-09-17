---
title:
- type: Text
  value: Boolean
---

# Boolean

**A value that is either true or false.**

**`@id`**: [`schema:Boolean`](https://schema.org/Boolean)

## Formats

The `Boolean` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding      | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | ------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss    |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🔷 Low loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🔷 Low loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss    |              | 🟢 Stable               |       |

## Bindings

The `Boolean` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Boolean.jsonld)
- [JSON Schema](https://stencila.dev/Boolean.schema.json)
- Python type [`Boolean`](https://github.com/stencila/stencila/blob/main/python/stencila/types/boolean.py)
- Rust type [`Boolean`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/boolean.rs)
- TypeScript type [`Boolean`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Boolean.ts)

## Source

This documentation was generated from [`Boolean.yaml`](https://github.com/stencila/stencila/blob/main/schema/Boolean.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).