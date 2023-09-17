---
title:
- type: Text
  value: Number
---

# Number

**A value that is a number**

**`@id`**: [`schema:Number`](https://schema.org/Number)

## Formats

The `Number` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `Number` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Number.jsonld)
- [JSON Schema](https://stencila.dev/Number.schema.json)
- Python type [`Number`](https://github.com/stencila/stencila/blob/main/python/stencila/types/number.py)
- Rust type [`Number`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/number.rs)
- TypeScript type [`Number`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Number.ts)

## Source

This documentation was generated from [`Number.yaml`](https://github.com/stencila/stencila/blob/main/schema/Number.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).