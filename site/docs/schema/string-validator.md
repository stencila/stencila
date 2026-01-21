---
title: String Validator
description: A schema specifying constraints on a string node.
---

A node will be valid against the schema if it is a string that
meets the schemas `minLength`, `maxLength` and `pattern` properties.
Analogous to the JSON Schema `string` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


# Properties

The `StringValidator` type has these properties:

| Name        | Description                                         | Type                      | Inherited from          |
| ----------- | --------------------------------------------------- | ------------------------- | ----------------------- |
| `id`        | The identifier for this item.                       | [`String`](./string.md)   | [`Entity`](./entity.md) |
| `minLength` | The minimum length for a string node.               | [`Integer`](./integer.md) | -                       |
| `maxLength` | The maximum length for a string node.               | [`Integer`](./integer.md) | -                       |
| `pattern`   | A regular expression that a string node must match. | [`String`](./string.md)   | -                       |

# Related

The `StringValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `StringValidator` type is represented in:

- [JSON-LD](https://stencila.org/StringValidator.jsonld)
- [JSON Schema](https://stencila.org/StringValidator.schema.json)
- Python class [`StringValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/string_validator.py)
- Rust struct [`StringValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/string_validator.rs)
- TypeScript class [`StringValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/StringValidator.ts)

# Source

This documentation was generated from [`StringValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/StringValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
