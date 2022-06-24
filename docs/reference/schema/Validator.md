# Validator

**A base for all validator types.**

The main purpose of this type is to obtain a generated union type for all validators.

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name | `@id`                                                 | Type   | Description                         | Inherited from      |
| ---- | ----------------------------------------------------- | ------ | ----------------------------------- | ------------------- |
| id   | [schema:id](https://schema.org/id)                    | string | The identifier for this item.       | [Entity](Entity.md) |
| meta | [stencila:meta](https://schema.stenci.la/meta.jsonld) | object | Metadata associated with this item. | [Entity](Entity.md) |

## Related

- Parent: [Entity](Entity.md)
- Descendants: [ArrayValidator](ArrayValidator.md), [BooleanValidator](BooleanValidator.md), [ConstantValidator](ConstantValidator.md), [EnumValidator](EnumValidator.md), [IntegerValidator](IntegerValidator.md), [NumberValidator](NumberValidator.md), [StringValidator](StringValidator.md), [TupleValidator](TupleValidator.md)

## Available as

- [JSON-LD](https://schema.stenci.la/Validator.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Validator.schema.json)
- Python [`class Validator`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Validator)
- TypeScript [`interface Validator`](https://stencila.github.io/schema/ts/docs/interfaces/validator.html)
- R [`class Validator`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Validator`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Validator.html)

## Source

This documentation was generated from [Validator.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/Validator.schema.yaml).
