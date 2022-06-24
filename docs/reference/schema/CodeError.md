# Code Error

**An error that occurred when parsing, compiling or executing a Code node.**

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name             | `@id`                                                                 | Type   | Description                                                                      | Inherited from            |
| ---------------- | --------------------------------------------------------------------- | ------ | -------------------------------------------------------------------------------- | ------------------------- |
| **errorMessage** | [stencila:errorMessage](https://schema.stenci.la/errorMessage.jsonld) | string | The error message or brief description of the error.                             | [CodeError](CodeError.md) |
| errorType        | [stencila:errorType](https://schema.stenci.la/errorType.jsonld)       | string | The type of error e.g. "SyntaxError", "ZeroDivisionError". See note [1](#notes). | [CodeError](CodeError.md) |
| id               | [schema:id](https://schema.org/id)                                    | string | The identifier for this item.                                                    | [Entity](Entity.md)       |
| meta             | [stencila:meta](https://schema.stenci.la/meta.jsonld)                 | object | Metadata associated with this item.                                              | [Entity](Entity.md)       |
| stackTrace       | [stencila:stackTrace](https://schema.stenci.la/stackTrace.jsonld)     | string | Stack trace leading up to the error.                                             | [CodeError](CodeError.md) |

## Notes

1. **errorType** : Many languages have the concept of alternative types of errors. For example, Python has various [classes of exceptions](https://docs.python.org/3/tutorial/errors.html). This property is intended to be used for storing these type names as additional information that maybe useful to the user attempting to resolve the error.

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/CodeError.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/CodeError.schema.json)
- Python [`class CodeError`](https://stencila.github.io/schema/python/docs/types.html#schema.types.CodeError)
- TypeScript [`interface CodeError`](https://stencila.github.io/schema/ts/docs/interfaces/codeerror.html)
- R [`class CodeError`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct CodeError`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.CodeError.html)

## Source

This documentation was generated from [CodeError.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/CodeError.schema.yaml).
