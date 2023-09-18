# Boolean

**A value that is either true or false.**

**`@id`**: [`schema:Boolean`](https://schema.org/Boolean)

## Formats

The `Boolean` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                            | Encoding      | Decoding     | Status                 | Notes |
| ------------------------------------------------------------------------------------------------- | ------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/HTML.md)             | 🔷 Low loss    |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JATS.md)             | 🔷 Low loss    |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Markdown.md)     | 🔷 Low loss    |              | 🚧 Under development    |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Plain text.md) | 🔷 Low loss    |              | 🟥 Alpha                |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON.md)             | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON5.md)           | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/YAML.md)             | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Debug.md)           | 🔷 Low loss    |              | 🟢 Stable               |       |

## Bindings

The `Boolean` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Boolean.jsonld)
- [JSON Schema](https://stencila.dev/Boolean.schema.json)
- Python type [`Boolean`](https://github.com/stencila/stencila/blob/main/python/stencila/types/boolean.py)
- Rust type [`Boolean`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/boolean.rs)
- TypeScript type [`Boolean`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Boolean.ts)

## Source

This documentation was generated from [`Boolean.yaml`](https://github.com/stencila/stencila/blob/main/schema/Boolean.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).