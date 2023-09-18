# Date Validator

**A validator specifying the constraints on a date.**

**`@id`**: `stencila:DateValidator`

## Properties

The `DateValidator` type has these properties:

| Name    | `@id`                                | Type                                                                                            | Description                           | Inherited from                                                                                                 |
| ------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item          | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)               |
| minimum | `stencila:minimum`                   | [`Date`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date.md)     | The inclusive lower limit for a date. | [`DateValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date-validator.md) |
| maximum | `stencila:maximum`                   | [`Date`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date.md)     | The inclusive upper limit for a date. | [`DateValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date-validator.md) |

## Related

The `DateValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `DateValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `DateValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/DateValidator.jsonld)
- [JSON Schema](https://stencila.dev/DateValidator.schema.json)
- Python class [`DateValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/date_validator.py)
- Rust struct [`DateValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date_validator.rs)
- TypeScript class [`DateValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/DateValidator.ts)

## Source

This documentation was generated from [`DateValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DateValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).