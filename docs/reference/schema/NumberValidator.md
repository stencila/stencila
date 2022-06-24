# Number Validator

**A validator specifying the constraints on a numeric node.**

A node will be valid if it is a number that meets the `maximum`, `multipleOf` etc properties. Analogous to the JSON Schema `number` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1). Note that the `IntegerValidator` type extends this validator with the additional constraint that the number have no fractional part.

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name             | `@id`                                                                         | Type   | Description                                                               | Inherited from                        |
| ---------------- | ----------------------------------------------------------------------------- | ------ | ------------------------------------------------------------------------- | ------------------------------------- |
| exclusiveMaximum | [stencila:exclusiveMaximum](https://schema.stenci.la/exclusiveMaximum.jsonld) | number | The exclusive upper limit for a numeric node. See note [1](#notes).       | [NumberValidator](NumberValidator.md) |
| exclusiveMinimum | [stencila:exclusiveMinimum](https://schema.stenci.la/exclusiveMinimum.jsonld) | number | The exclusive lower limit for a numeric node. See note [2](#notes).       | [NumberValidator](NumberValidator.md) |
| id               | [schema:id](https://schema.org/id)                                            | string | The identifier for this item.                                             | [Entity](Entity.md)                   |
| maximum          | [stencila:maximum](https://schema.stenci.la/maximum.jsonld)                   | number | The inclusive upper limit for a numeric node. See note [3](#notes).       | [NumberValidator](NumberValidator.md) |
| meta             | [stencila:meta](https://schema.stenci.la/meta.jsonld)                         | object | Metadata associated with this item.                                       | [Entity](Entity.md)                   |
| minimum          | [stencila:minimum](https://schema.stenci.la/minimum.jsonld)                   | number | The inclusive lower limit for a numeric node. See note [4](#notes).       | [NumberValidator](NumberValidator.md) |
| multipleOf       | [stencila:multipleOf](https://schema.stenci.la/multipleOf.jsonld)             | number | A number that a numeric node must be a multiple of. See note [5](#notes). | [NumberValidator](NumberValidator.md) |

## Notes

1. **exclusiveMaximum** : A number is valid only if it has a value less than (not equal to) `exclusiveMaximum`.
2. **exclusiveMinimum** : A number is valid only if it has a value greater than (not equal to) `exclusiveMinimum`.
3. **maximum** : A number is valid if it is less than, or exactly equal to, `maximum`.
4. **minimum** : A number is valid if it is greater than, or exactly equal to, `minimum`.
5. **multipleOf** : A number is valid only if division by this value results in an integer.

## Related

- Parent: [Validator](Validator.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/NumberValidator.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/NumberValidator.schema.json)
- Python [`class NumberValidator`](https://stencila.github.io/schema/python/docs/types.html#schema.types.NumberValidator)
- TypeScript [`interface NumberValidator`](https://stencila.github.io/schema/ts/docs/interfaces/numbervalidator.html)
- R [`class NumberValidator`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct NumberValidator`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.NumberValidator.html)

## Source

This documentation was generated from [NumberValidator.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/NumberValidator.schema.yaml).
