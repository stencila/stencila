# String Validator

**A schema specifying constraints on a string node.**

A node will be valid against the schema if it is a string that meets the schemas `minLength`, `maxLength` and `pattern` properties. Analogous to the JSON Schema `string` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name      | `@id`                                                           | Type    | Description                                                               | Inherited from                        |
| --------- | --------------------------------------------------------------- | ------- | ------------------------------------------------------------------------- | ------------------------------------- |
| id        | [schema:id](https://schema.org/id)                              | string  | The identifier for this item.                                             | [Entity](Entity.md)                   |
| maxLength | [stencila:maxLength](https://schema.stenci.la/maxLength.jsonld) | integer | The maximum length for a string node. See note [1](#notes).               | [StringValidator](StringValidator.md) |
| meta      | [stencila:meta](https://schema.stenci.la/meta.jsonld)           | object  | Metadata associated with this item.                                       | [Entity](Entity.md)                   |
| minLength | [stencila:minLength](https://schema.stenci.la/minLength.jsonld) | integer | The minimum length for a string node. See note [2](#notes).               | [StringValidator](StringValidator.md) |
| pattern   | [stencila:pattern](https://schema.stenci.la/pattern.jsonld)     | string  | A regular expression that a string node must match. See note [3](#notes). | [StringValidator](StringValidator.md) |

## Notes

1. **maxLength** : A string is valid if its length is less than, or exactly equal to, `maxLength`.
2. **minLength** : A string is valid if its length is greater than, or exactly equal to, `minLength`.
3. **pattern** : A string is valid if it is matched by the regular expression.

## Related

- Parent: [Validator](Validator.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/StringValidator.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/StringValidator.schema.json)
- Python [`class StringValidator`](https://stencila.github.io/schema/python/docs/types.html#schema.types.StringValidator)
- TypeScript [`interface StringValidator`](https://stencila.github.io/schema/ts/docs/interfaces/stringvalidator.html)
- R [`class StringValidator`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct StringValidator`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.StringValidator.html)

## Source

This documentation was generated from [StringValidator.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/StringValidator.schema.yaml).
