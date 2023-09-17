---
title:
- type: Text
  value: BooleanValidator
---

# Boolean Validator

**A schema specifying that a node must be a boolean value.**

A node will be valid against this schema if it is either `true` or `false.
Analogous to the JSON Schema `boolean` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


**`@id`**: `stencila:BooleanValidator`

## Properties

The `BooleanValidator` type has these properties:

| Name | `@id`                                | Type                                                               | Description                  | Inherited from                                                      |
| ---- | ------------------------------------ | ------------------------------------------------------------------ | ---------------------------- | ------------------------------------------------------------------- |
| id   | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |

## Related

The `BooleanValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `BooleanValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `BooleanValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/BooleanValidator.jsonld)
- [JSON Schema](https://stencila.dev/BooleanValidator.schema.json)
- Python class [`BooleanValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/boolean_validator.py)
- Rust struct [`BooleanValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/boolean_validator.rs)
- TypeScript class [`BooleanValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/BooleanValidator.ts)

## Source

This documentation was generated from [`BooleanValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/BooleanValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).