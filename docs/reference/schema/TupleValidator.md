# Tuple Validator

**A validator specifying constraints on an array of heterogeneous items.**

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name  | `@id`                                                        | Type                                         | Description                                                                             | Inherited from                      |
| ----- | ------------------------------------------------------------ | -------------------------------------------- | --------------------------------------------------------------------------------------- | ----------------------------------- |
| id    | [schema:id](https://schema.org/id)                           | string                                       | The identifier for this item.                                                           | [Entity](Entity.md)                 |
| items | [schema:itemListElement](https://schema.org/itemListElement) | Array of [ValidatorTypes](ValidatorTypes.md) | An array of validators specifying the constraints on each successive item in the array. | [TupleValidator](TupleValidator.md) |
| meta  | [stencila:meta](https://schema.stenci.la/meta.jsonld)        | object                                       | Metadata associated with this item.                                                     | [Entity](Entity.md)                 |

## Related

- Parent: [Validator](Validator.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/TupleValidator.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/TupleValidator.schema.json)
- Python [`class TupleValidator`](https://stencila.github.io/schema/python/docs/types.html#schema.types.TupleValidator)
- TypeScript [`interface TupleValidator`](https://stencila.github.io/schema/ts/docs/interfaces/tuplevalidator.html)
- R [`class TupleValidator`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct TupleValidator`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.TupleValidator.html)

## Source

This documentation was generated from [TupleValidator.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/TupleValidator.schema.yaml).
