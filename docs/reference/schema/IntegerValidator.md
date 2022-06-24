# Integer Validator

**A validator specifying the constraints on an integer node.**

A node will be valid if it is a number with no fractional part and meets any additional constraints, such as `multipleOf`, specified in the validator. Analogous to the JSON Schema `integer` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name | `@id`                                                 | Type   | Description                         | Inherited from      |
| ---- | ----------------------------------------------------- | ------ | ----------------------------------- | ------------------- |
| id   | [schema:id](https://schema.org/id)                    | string | The identifier for this item.       | [Entity](Entity.md) |
| meta | [stencila:meta](https://schema.stenci.la/meta.jsonld) | object | Metadata associated with this item. | [Entity](Entity.md) |

## Related

- Parent: [Validator](Validator.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/IntegerValidator.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/IntegerValidator.schema.json)
- Python [`class IntegerValidator`](https://stencila.github.io/schema/python/docs/types.html#schema.types.IntegerValidator)
- TypeScript [`interface IntegerValidator`](https://stencila.github.io/schema/ts/docs/interfaces/integervalidator.html)
- R [`class IntegerValidator`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct IntegerValidator`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.IntegerValidator.html)

## Source

This documentation was generated from [IntegerValidator.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/IntegerValidator.schema.yaml).
