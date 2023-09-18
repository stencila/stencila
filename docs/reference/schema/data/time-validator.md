# Time Validator

**A validator specifying the constraints on a time.**

**`@id`**: `stencila:TimeValidator`

## Properties

The `TimeValidator` type has these properties:

| Name    | `@id`                                | Type                                                                                            | Description                           | Inherited from                                                                                                 |
| ------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item          | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)               |
| minimum | `stencila:minimum`                   | [`Time`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time.md)     | The inclusive lower limit for a time. | [`TimeValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time-validator.md) |
| maximum | `stencila:maximum`                   | [`Time`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time.md)     | The inclusive upper limit for a time. | [`TimeValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time-validator.md) |

## Related

The `TimeValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `TimeValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `TimeValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TimeValidator.jsonld)
- [JSON Schema](https://stencila.dev/TimeValidator.schema.json)
- Python class [`TimeValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/time_validator.py)
- Rust struct [`TimeValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/time_validator.rs)
- TypeScript class [`TimeValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TimeValidator.ts)

## Source

This documentation was generated from [`TimeValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimeValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).