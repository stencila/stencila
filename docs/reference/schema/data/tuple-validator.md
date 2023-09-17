---
title:
- type: Text
  value: TupleValidator
---

# Tuple Validator

**A validator specifying constraints on an array of heterogeneous items.**

**`@id`**: `stencila:TupleValidator`

## Properties

The `TupleValidator` type has these properties:

| Name  | `@id`                                                          | Type                                                                      | Description                                                                             | Inherited from                                                                      |
| ----- | -------------------------------------------------------------- | ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| id    | [`schema:id`](https://schema.org/id)                           | [`String`](https://stencila.dev/docs/reference/schema/data/string)        | The identifier for this item                                                            | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                 |
| items | [`schema:itemListElement`](https://schema.org/itemListElement) | [`Validator`](https://stencila.dev/docs/reference/schema/data/validator)* | An array of validators specifying the constraints on each successive item in the array. | [`TupleValidator`](https://stencila.dev/docs/reference/schema/data/tuple-validator) |

## Related

The `TupleValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `TupleValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `TupleValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TupleValidator.jsonld)
- [JSON Schema](https://stencila.dev/TupleValidator.schema.json)
- Python class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/tuple_validator.py)
- Rust struct [`TupleValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/tuple_validator.rs)
- TypeScript class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TupleValidator.ts)

## Source

This documentation was generated from [`TupleValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TupleValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).