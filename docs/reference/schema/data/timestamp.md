# Timestamp

**A value that represents a point in time.**

**`@id`**: [`schema:Timestamp`](https://schema.org/Timestamp)

## Properties

The `Timestamp` type has these properties:

| Name       | Aliases                  | `@id`                                      | Type                                                                                                 | Description                                                                      | Inherited from                                                                                   |
| ---------- | ------------------------ | ------------------------------------------ | ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`       | -                        | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)      | The identifier for this item.                                                    | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `value`    | -                        | [`schema:value`](https://schema.org/value) | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)    | The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z). | -                                                                                                |
| `timeUnit` | `time-unit`, `time_unit` | `stencila:timeUnit`                        | [`TimeUnit`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time-unit.md) | The time unit that the `value` represents.                                       | -                                                                                                |

## Related

The `Timestamp` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Timestamp` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes                                                                                                                               |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |              | 🚧 Under development    |                                                                                                                                     |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              | 🟢 No loss        | 🟢 No loss    | 🚧 Under development    | Encoded as [`<timestamp>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/timestamp.html) using special function |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                                                                     |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                                                                     |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                     |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                     |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss    | 🔶 Beta                 |                                                                                                                                     |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                     |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                     |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                                     |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |                                                                                                                                     |

## Bindings

The `Timestamp` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Timestamp.jsonld)
- [JSON Schema](https://stencila.org/Timestamp.schema.json)
- Python class [`Timestamp`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/timestamp.py)
- Rust struct [`Timestamp`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp.rs)
- TypeScript class [`Timestamp`](https://github.com/stencila/stencila/blob/main/ts/src/types/Timestamp.ts)

## Source

This documentation was generated from [`Timestamp.yaml`](https://github.com/stencila/stencila/blob/main/schema/Timestamp.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).