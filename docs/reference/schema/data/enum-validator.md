---
title:
- type: Text
  value: EnumValidator
---

# Enum Validator

**A schema specifying that a node must be one of several values.**

Analogous to the JSON Schema [`enum` keyword](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.2).

**`@id`**: `stencila:EnumValidator`

## Properties

The `EnumValidator` type has these properties:

| Name   | `@id`                                | Type                                                               | Description                                            | Inherited from                                                                    |
| ------ | ------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------------------ | --------------------------------------------------------------------------------- |
| id     | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item                           | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)               |
| values | `stencila:values`                    | [`Node`](https://stencila.dev/docs/reference/schema/other/node)*   | A node is valid if it is equal to any of these values. | [`EnumValidator`](https://stencila.dev/docs/reference/schema/data/enum-validator) |

## Related

The `EnumValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `EnumValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `EnumValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/EnumValidator.jsonld)
- [JSON Schema](https://stencila.dev/EnumValidator.schema.json)
- Python class [`EnumValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/enum_validator.py)
- Rust struct [`EnumValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/enum_validator.rs)
- TypeScript class [`EnumValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/EnumValidator.ts)

## Source

This documentation was generated from [`EnumValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/EnumValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).