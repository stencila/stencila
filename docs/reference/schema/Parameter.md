# Parameter

**A parameter of a document or function.**

This schema type is marked as **experimental** 🧪 and is subject to change.

## Properties

| Name          | `@id`                                                                   | Type                                | Description                                                                                           | Inherited from            |
| ------------- | ----------------------------------------------------------------------- | ----------------------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------- |
| **name**      | [schema:name](https://schema.org/name)                                  | string                              | The name of the parameter. See note [1](#notes).                                                      | [Parameter](Parameter.md) |
| default       | [schema:defaultValue](https://schema.org/defaultValue)                  | [Node](Node.md)                     | The default value of the parameter.                                                                   | [Parameter](Parameter.md) |
| executeDigest | [stencila:executeDigest](https://schema.stenci.la/executeDigest.jsonld) | string                              | The SHA-256 digest of the `value` property the last time the node was executed. See note [2](#notes). | [Parameter](Parameter.md) |
| id            | [schema:id](https://schema.org/id)                                      | string                              | The identifier for this item.                                                                         | [Entity](Entity.md)       |
| isExtensible  | [stencila:isExtensible](https://schema.stenci.la/isExtensible.jsonld)   | boolean                             | Indicates that this parameter is variadic and can accept multiple named arguments.                    | [Parameter](Parameter.md) |
| isRequired    | [schema:valueRequired](https://schema.org/valueRequired)                | boolean                             | Is this parameter required, if not it should have a default or default is assumed to be null.         | [Parameter](Parameter.md) |
| isVariadic    | [stencila:isVariadic](https://schema.stenci.la/isVariadic.jsonld)       | boolean                             | Indicates that this parameter is variadic and can accept multiple arguments.                          | [Parameter](Parameter.md) |
| meta          | [stencila:meta](https://schema.stenci.la/meta.jsonld)                   | object                              | Metadata associated with this item.                                                                   | [Entity](Entity.md)       |
| validator     | [stencila:validator](https://schema.stenci.la/validator.jsonld)         | [ValidatorTypes](ValidatorTypes.md) | The validator that the value is validated against.                                                    | [Parameter](Parameter.md) |
| value         | [schema:value](https://schema.org/value)                                | [Node](Node.md)                     | The current value of the parameter.                                                                   | [Parameter](Parameter.md) |

## Notes

1. **name** : This regex allows for snake_case and camelCase names but excludes PascalCase for names.
2. **executeDigest** : Used to determine whether it is necessary to re-execute the node (i.e to assign a variable within the document session having `name` and `value`).

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Parameter.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Parameter.schema.json)
- Python [`class Parameter`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Parameter)
- TypeScript [`interface Parameter`](https://stencila.github.io/schema/ts/docs/interfaces/parameter.html)
- R [`class Parameter`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Parameter`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Parameter.html)

## Source

This documentation was generated from [Parameter.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Parameter.schema.yaml).
