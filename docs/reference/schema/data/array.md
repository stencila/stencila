---
title:
- type: Text
  value: Array
---

# Array

**A value comprised of other primitive nodes**

Note that the items in the array are restricted to primitive node
types including `Array` (ie. an `Array` as an item of another `Array`) and `Object`.


**`@id`**: `stencila:Array`

## Formats

The `Array` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Array` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Array.jsonld)
- [JSON Schema](https://stencila.dev/Array.schema.json)
- Python type [`Array`](https://github.com/stencila/stencila/blob/main/python/stencila/types/array.py)
- Rust type [`Array`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/array.rs)
- TypeScript type [`Array`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Array.ts)

## Source

This documentation was generated from [`Array.yaml`](https://github.com/stencila/stencila/blob/main/schema/Array.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).