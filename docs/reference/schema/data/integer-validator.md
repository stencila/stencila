---
title:
- type: Text
  value: IntegerValidator
---

# Integer Validator

**A validator specifying the constraints on an integer node.**

A node will be valid if it is a number with no fractional part and meets any additional constraints,
such as `multipleOf`, specified in the validator.
Analogous to the JSON Schema `integer` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


**`@id`**: `stencila:IntegerValidator`

## Properties

The `IntegerValidator` type has these properties:

| Name             | `@id`                                | Type                                                               | Description                                         | Inherited from                                                                        |
| ---------------- | ------------------------------------ | ------------------------------------------------------------------ | --------------------------------------------------- | ------------------------------------------------------------------------------------- |
| id               | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item                        | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                   |
| minimum          | `stencila:minimum`                   | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | The inclusive lower limit for a numeric node.       | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |
| exclusiveMinimum | `stencila:exclusiveMinimum`          | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | The exclusive lower limit for a numeric node.       | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |
| maximum          | `stencila:maximum`                   | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | The inclusive upper limit for a numeric node.       | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |
| exclusiveMaximum | `stencila:exclusiveMaximum`          | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | The exclusive upper limit for a numeric node.       | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |
| multipleOf       | `stencila:multipleOf`                | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | A number that a numeric node must be a multiple of. | [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator) |

## Related

The `IntegerValidator` type is related to these types:

- Parents: [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator)
- Children: none

## Formats

The `IntegerValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `IntegerValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/IntegerValidator.jsonld)
- [JSON Schema](https://stencila.dev/IntegerValidator.schema.json)
- Python class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/integer_validator.py)
- Rust struct [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer_validator.rs)
- TypeScript class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/IntegerValidator.ts)

## Source

This documentation was generated from [`IntegerValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/IntegerValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).