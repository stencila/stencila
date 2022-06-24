# Boolean Validator

**A schema specifying that a node must be a boolean value.**

A node will be valid against this schema if it is either `true` or `false. Analogous to the JSON Schema `boolean` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).

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

- [JSON-LD](https://schema.stenci.la/BooleanValidator.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/BooleanValidator.schema.json)
- Python [`class BooleanValidator`](https://stencila.github.io/schema/python/docs/types.html#schema.types.BooleanValidator)
- TypeScript [`interface BooleanValidator`](https://stencila.github.io/schema/ts/docs/interfaces/booleanvalidator.html)
- R [`class BooleanValidator`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct BooleanValidator`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.BooleanValidator.html)

## Source

This documentation was generated from [BooleanValidator.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/BooleanValidator.schema.yaml).
