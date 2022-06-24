# Function

**A function with a name, which might take Parameters and return a value of a certain type.**

This schema type is marked as **experimental** 🧪 and is subject to change.

## Properties

| Name       | `@id`                                                             | Type                                | Description                                     | Inherited from          |
| ---------- | ----------------------------------------------------------------- | ----------------------------------- | ----------------------------------------------- | ----------------------- |
| id         | [schema:id](https://schema.org/id)                                | string                              | The identifier for this item.                   | [Entity](Entity.md)     |
| meta       | [stencila:meta](https://schema.stenci.la/meta.jsonld)             | object                              | Metadata associated with this item.             | [Entity](Entity.md)     |
| name       | [schema:name](https://schema.org/name)                            | string                              | The name of the function. See note [1](#notes). | [Function](Function.md) |
| parameters | [stencila:parameters](https://schema.stenci.la/parameters.jsonld) | Array of [Parameter](Parameter.md)  | The parameters of the function.                 | [Function](Function.md) |
| returns    | [stencila:returns](https://schema.stenci.la/returns.jsonld)       | [ValidatorTypes](ValidatorTypes.md) | The return type of the function.                | [Function](Function.md) |

## Notes

1. **name** : The name property is not required; this allows for anonymous functions (although these are not yet implemented in Stencila interpreters). The regex allows for snake_case and camelCase names but excludes PascalCase for parameter names.

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Function.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Function.schema.json)
- Python [`class Function`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Function)
- TypeScript [`interface Function`](https://stencila.github.io/schema/ts/docs/interfaces/function.html)
- R [`class Function`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Function`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Function.html)

## Source

This documentation was generated from [Function.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Function.schema.yaml).
