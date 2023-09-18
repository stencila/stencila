# Array

**A value comprised of other primitive nodes**

Note that the items in the array are restricted to primitive node
types including `Array` (ie. an `Array` as an item of another `Array`) and `Object`.


**`@id`**: `stencila:Array`

## Formats

The `Array` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                            | Encoding       | Decoding     | Status                 | Notes |
| ------------------------------------------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/HTML.md)             | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JATS.md)             | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Markdown.md)     | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Plain text.md) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON5.md)           | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/YAML.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Debug.md)           | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Array` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Array.jsonld)
- [JSON Schema](https://stencila.dev/Array.schema.json)
- Python type [`Array`](https://github.com/stencila/stencila/blob/main/python/stencila/types/array.py)
- Rust type [`Array`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/array.rs)
- TypeScript type [`Array`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Array.ts)

## Source

This documentation was generated from [`Array.yaml`](https://github.com/stencila/stencila/blob/main/schema/Array.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).