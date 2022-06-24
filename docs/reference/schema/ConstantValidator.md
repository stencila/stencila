# Constant Validator

**A validator specifying a constant value that a node must have.**

A node will be valid against this validator if it is equal to the `value` property. Analogous to the JSON Schema [`const` keyword](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.3).

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name  | `@id`                                                 | Type            | Description                         | Inherited from                            |
| ----- | ----------------------------------------------------- | --------------- | ----------------------------------- | ----------------------------------------- |
| id    | [schema:id](https://schema.org/id)                    | string          | The identifier for this item.       | [Entity](Entity.md)                       |
| meta  | [stencila:meta](https://schema.stenci.la/meta.jsonld) | object          | Metadata associated with this item. | [Entity](Entity.md)                       |
| value | [schema:value](https://schema.org/value)              | [Node](Node.md) | The value that the node must have.  | [ConstantValidator](ConstantValidator.md) |

## Related

- Parent: [Validator](Validator.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/ConstantValidator.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/ConstantValidator.schema.json)
- Python [`class ConstantValidator`](https://stencila.github.io/schema/python/docs/types.html#schema.types.ConstantValidator)
- TypeScript [`interface ConstantValidator`](https://stencila.github.io/schema/ts/docs/interfaces/constantvalidator.html)
- R [`class ConstantValidator`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct ConstantValidator`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.ConstantValidator.html)

## Source

This documentation was generated from [ConstantValidator.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/ConstantValidator.schema.yaml).
