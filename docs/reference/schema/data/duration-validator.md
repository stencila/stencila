# Duration Validator

**A validator specifying the constraints on a duration.**

**`@id`**: `stencila:DurationValidator`

## Properties

The `DurationValidator` type has these properties:

| Name      | `@id`                                | Type                                                                                                  | Description                                | Inherited from                                                                                                         |
| --------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| id        | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)       | The identifier for this item               | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)                       |
| timeUnits | `stencila:timeUnits`                 | [`TimeUnit`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time-unit.md)* | The time units that the duration can have. | [`DurationValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration-validator.md) |
| minimum   | `stencila:minimum`                   | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)   | The inclusive lower limit for a duration.  | [`DurationValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration-validator.md) |
| maximum   | `stencila:maximum`                   | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)   | The inclusive upper limit for a duration.  | [`DurationValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration-validator.md) |

## Related

The `DurationValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `DurationValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `DurationValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/DurationValidator.jsonld)
- [JSON Schema](https://stencila.dev/DurationValidator.schema.json)
- Python class [`DurationValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/duration_validator.py)
- Rust struct [`DurationValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/duration_validator.rs)
- TypeScript class [`DurationValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/DurationValidator.ts)

## Source

This documentation was generated from [`DurationValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DurationValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).