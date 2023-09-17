---
title:
- type: Text
  value: Object
---

# Object

**A value comprised of keyed primitive nodes.**

Note that keys are strings and values are restricted to primitive node
types including `Object` (ie. an `Object` as a value of another `Object`) and `Array`.


**`@id`**: `stencila:Object`

## Formats

The `Object` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Object` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Object.jsonld)
- [JSON Schema](https://stencila.dev/Object.schema.json)
- Python type [`Object`](https://github.com/stencila/stencila/blob/main/python/stencila/types/object.py)
- Rust type [`Object`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/object.rs)
- TypeScript type [`Object`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Object.ts)

## Source

This documentation was generated from [`Object.yaml`](https://github.com/stencila/stencila/blob/main/schema/Object.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).