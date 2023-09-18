# Date Time

**A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.**

**`@id`**: [`schema:DateTime`](https://schema.org/DateTime)

## Properties

The `DateTime` type has these properties:

| Name  | `@id`                                      | Type                                                                                            | Description                     | Inherited from                                                                                       |
| ----- | ------------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------- | ---------------------------------------------------------------------------------------------------- |
| id    | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item    | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)     |
| value | [`schema:value`](https://schema.org/value) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The date as an ISO 8601 string. | [`DateTime`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date-time.md) |

## Related

The `DateTime` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `DateTime` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding       | Decoding     | Status                 | Notes |
| --------------------------------------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         |                |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `DateTime` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/DateTime.jsonld)
- [JSON Schema](https://stencila.dev/DateTime.schema.json)
- Python class [`DateTime`](https://github.com/stencila/stencila/blob/main/python/stencila/types/date_time.py)
- Rust struct [`DateTime`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date_time.rs)
- TypeScript class [`DateTime`](https://github.com/stencila/stencila/blob/main/typescript/src/types/DateTime.ts)

## Source

This documentation was generated from [`DateTime.yaml`](https://github.com/stencila/stencila/blob/main/schema/DateTime.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).