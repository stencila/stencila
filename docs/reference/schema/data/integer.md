# Integer

**A value that is a integer.**

**`@id`**: [`schema:Integer`](https://schema.org/Integer)

## Formats

The `Integer` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding      | Decoding      | Status                 | Notes |
| -------------------------------------------------------------------------------------------------- | ------------- | ------------- | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss    |               | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              | 🔷 Low loss    |               | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | 🔷 Low loss    | 🔷 Low loss    | ⚠️ Alpha               |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | 🔷 Low loss    |               | ⚠️ Alpha               |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss     | 🟢 No loss     | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss     | 🟢 No loss     | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss     | 🟢 No loss     | 🟢 Stable               |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss     | 🟢 No loss     | 🟢 Stable               |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss     | 🟢 No loss     | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss    |               | 🟢 Stable               |       |

## Bindings

The `Integer` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Integer.jsonld)
- [JSON Schema](https://stencila.dev/Integer.schema.json)
- Python type [`Integer`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/integer.py)
- Rust type [`Integer`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer.rs)
- TypeScript type [`Integer`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Integer.ts)

## Source

This documentation was generated from [`Integer.yaml`](https://github.com/stencila/stencila/blob/main/schema/Integer.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).