# Enum Validator

**A schema specifying that a node must be one of several values.**

Analogous to the JSON Schema [`enum` keyword](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.2).

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name   | `@id`                                                     | Type                     | Description                                            | Inherited from                    |
| ------ | --------------------------------------------------------- | ------------------------ | ------------------------------------------------------ | --------------------------------- |
| id     | [schema:id](https://schema.org/id)                        | string                   | The identifier for this item.                          | [Entity](Entity.md)               |
| meta   | [stencila:meta](https://schema.stenci.la/meta.jsonld)     | object                   | Metadata associated with this item.                    | [Entity](Entity.md)               |
| values | [stencila:values](https://schema.stenci.la/values.jsonld) | Array of [Node](Node.md) | A node is valid if it is equal to any of these values. | [EnumValidator](EnumValidator.md) |

## Related

- Parent: [Validator](Validator.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/EnumValidator.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/EnumValidator.schema.json)
- Python [`class EnumValidator`](https://stencila.github.io/schema/python/docs/types.html#schema.types.EnumValidator)
- TypeScript [`interface EnumValidator`](https://stencila.github.io/schema/ts/docs/interfaces/enumvalidator.html)
- R [`class EnumValidator`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct EnumValidator`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.EnumValidator.html)

## Source

This documentation was generated from [EnumValidator.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/EnumValidator.schema.yaml).
