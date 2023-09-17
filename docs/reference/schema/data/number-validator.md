---
title:
- type: Text
  value: NumberValidator
---

# Number Validator

**A validator specifying the constraints on a numeric node.**

A node will be valid if it is a number that meets the `maximum`, `multipleOf` etc properties.
Analogous to the JSON Schema `number` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).
Note that the `IntegerValidator` type extends this validator with the additional
constraint that the number have no fractional part.


**`@id`**: `stencila:NumberValidator`

## Properties

The `NumberValidator` type has these properties:

| Name             | `@id`                                | Type                                                               | Description                                         | Inherited from                                                                        |
| ---------------- | ------------------------------------ | ------------------------------------------------------------------ | --------------------------------------------------- | ------------------------------------------------------------------------------------- |
| id               | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item                        | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                   |
| minimum          | `stencila:minimum`                   | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | The inclusive lower limit for a numeric node.       | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |
| exclusiveMinimum | `stencila:exclusiveMinimum`          | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | The exclusive lower limit for a numeric node.       | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |
| maximum          | `stencila:maximum`                   | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | The inclusive upper limit for a numeric node.       | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |
| exclusiveMaximum | `stencila:exclusiveMaximum`          | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | The exclusive upper limit for a numeric node.       | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |
| multipleOf       | `stencila:multipleOf`                | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | A number that a numeric node must be a multiple of. | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |

## Related

The `NumberValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: [`IntegerValidator`](https://stencila.dev/docs/reference/schema/data/integer-validator)

## Formats

The `NumberValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `NumberValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/NumberValidator.jsonld)
- [JSON Schema](https://stencila.dev/NumberValidator.schema.json)
- Python class [`NumberValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/number_validator.py)
- Rust struct [`NumberValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/number_validator.rs)
- TypeScript class [`NumberValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/NumberValidator.ts)

## Source

This documentation was generated from [`NumberValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/NumberValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).