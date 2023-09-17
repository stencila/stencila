---
title:
- type: Text
  value: StringValidator
---

# String Validator

**A schema specifying constraints on a string node.**

A node will be valid against the schema if it is a string that
meets the schemas `minLength`, `maxLength` and `pattern` properties.
Analogous to the JSON Schema `string` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


**`@id`**: `stencila:StringValidator`

## Properties

The `StringValidator` type has these properties:

| Name      | `@id`                                | Type                                                                 | Description                                         | Inherited from                                                                        |
| --------- | ------------------------------------ | -------------------------------------------------------------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------- |
| id        | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The identifier for this item                        | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                   |
| minLength | `stencila:minLength`                 | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer) | The minimum length for a string node.               | [`StringValidator`](https://stencila.dev/docs/reference/schema/data/string-validator) |
| maxLength | `stencila:maxLength`                 | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer) | The maximum length for a string node.               | [`StringValidator`](https://stencila.dev/docs/reference/schema/data/string-validator) |
| pattern   | `stencila:pattern`                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | A regular expression that a string node must match. | [`StringValidator`](https://stencila.dev/docs/reference/schema/data/string-validator) |

## Related

The `StringValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `StringValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `StringValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/StringValidator.jsonld)
- [JSON Schema](https://stencila.dev/StringValidator.schema.json)
- Python class [`StringValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/string_validator.py)
- Rust struct [`StringValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/string_validator.rs)
- TypeScript class [`StringValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/StringValidator.ts)

## Source

This documentation was generated from [`StringValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/StringValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).