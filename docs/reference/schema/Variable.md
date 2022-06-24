# Variable

**A variable representing a name / value pair.**

This schema type is marked as **experimental** 🧪 and is subject to change.

## Properties

| Name       | `@id`                                                           | Type                                | Description                                                                   | Inherited from          |
| ---------- | --------------------------------------------------------------- | ----------------------------------- | ----------------------------------------------------------------------------- | ----------------------- |
| **name**   | [schema:name](https://schema.org/name)                          | string                              | The name of the variable. See note [1](#notes).                               | [Variable](Variable.md) |
| id         | [schema:id](https://schema.org/id)                              | string                              | The identifier for this item.                                                 | [Entity](Entity.md)     |
| isReadonly | [schema:readonlyValue](https://schema.org/readonlyValue)        | boolean                             | Whether or not a property is mutable. Default is false. See note [2](#notes). | [Variable](Variable.md) |
| meta       | [stencila:meta](https://schema.stenci.la/meta.jsonld)           | object                              | Metadata associated with this item.                                           | [Entity](Entity.md)     |
| validator  | [stencila:validator](https://schema.stenci.la/validator.jsonld) | [ValidatorTypes](ValidatorTypes.md) | The validator that the value is validated against.                            | [Variable](Variable.md) |
| value      | [schema:value](https://schema.org/value)                        | [Node](Node.md)                     | The value of the variable.                                                    | [Variable](Variable.md) |

## Notes

1. **name** : This regex allows for snake_case and camelCase names but excludes PascalCase for names.
2. **isReadonly** : If `isReadonly` is `true` and `value` is defined then changes to `value` should not be allowed.

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Variable.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Variable.schema.json)
- Python [`class Variable`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Variable)
- TypeScript [`interface Variable`](https://stencila.github.io/schema/ts/docs/interfaces/variable.html)
- R [`class Variable`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Variable`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Variable.html)

## Source

This documentation was generated from [Variable.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Variable.schema.yaml).
