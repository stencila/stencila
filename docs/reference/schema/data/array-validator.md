---
title:
- type: Text
  value: ArrayValidator
---

# Array Validator

**A validator specifying constraints on an array node.**

**`@id`**: `stencila:ArrayValidator`

## Properties

The `ArrayValidator` type has these properties:

| Name           | `@id`                                | Type                                                                     | Description                                                                                 | Inherited from                                                                      |
| -------------- | ------------------------------------ | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| id             | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)       | The identifier for this item                                                                | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                 |
| itemsNullable  | `stencila:itemsNullable`             | [`Boolean`](https://stencila.dev/docs/reference/schema/data/boolean)     | Whether items can have the value `Node::Null`                                               | [`ArrayValidator`](https://stencila.dev/docs/reference/schema/data/array-validator) |
| itemsValidator | `stencila:itemsValidator`            | [`Validator`](https://stencila.dev/docs/reference/schema/data/validator) | Another validator node specifying the constraints on all items in the array.                | [`ArrayValidator`](https://stencila.dev/docs/reference/schema/data/array-validator) |
| contains       | `stencila:contains`                  | [`Validator`](https://stencila.dev/docs/reference/schema/data/validator) | An array node is valid if at least one of its items is valid against the `contains` schema. | [`ArrayValidator`](https://stencila.dev/docs/reference/schema/data/array-validator) |
| minItems       | `stencila:minItems`                  | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)     | An array node is valid if its size is greater than, or equal to, this value.                | [`ArrayValidator`](https://stencila.dev/docs/reference/schema/data/array-validator) |
| maxItems       | `stencila:maxItems`                  | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)     | An array node is valid if its size is less than, or equal to, this value.                   | [`ArrayValidator`](https://stencila.dev/docs/reference/schema/data/array-validator) |
| uniqueItems    | `stencila:uniqueItems`               | [`Boolean`](https://stencila.dev/docs/reference/schema/data/boolean)     | A flag to indicate that each value in the array should be unique.                           | [`ArrayValidator`](https://stencila.dev/docs/reference/schema/data/array-validator) |

## Related

The `ArrayValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `ArrayValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ArrayValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ArrayValidator.jsonld)
- [JSON Schema](https://stencila.dev/ArrayValidator.schema.json)
- Python class [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/array_validator.py)
- Rust struct [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/array_validator.rs)
- TypeScript class [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ArrayValidator.ts)

## Source

This documentation was generated from [`ArrayValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/ArrayValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).