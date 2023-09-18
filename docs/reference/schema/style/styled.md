# Styled

**An abstract base class for a document node that has styling applied to it and/or its content**

This class is very similar to the `Math` abstract base class but has `styleLanguage` instead
of `mathLanguage` and compiled `css` instead of `mathml`.

Note also that `styleLanguage` is optional.


**`@id`**: `stencila:Styled`

This type is marked as experimental and is likely to change.

## Properties

The `Styled` type has these properties:

| Name          | `@id`                                | Type                                                                                                               | Description                                                                | Inherited from                                                                                   |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The identifier for this item                                               | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| code          | `stencila:code`                      | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                        | The code of the equation in the `styleLanguage`.                           | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| styleLanguage | `stencila:styleLanguage`             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The language used for the style specification e.g. css, tailwind, classes. | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md) | A digest of the `code` and `styleLanguage`.                                | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| errors        | `stencila:errors`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                   | Errors that occurred when transpiling the `code`.                          | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| css           | `stencila:css`                       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | A Cascading Style Sheet (CSS) transpiled from the `code` property.         | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| classes       | `stencila:classes`                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                   | A list of class names associated with the node                             | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |

## Related

The `Styled` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`Division`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/division.md), [`Span`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/span.md)

## Bindings

The `Styled` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Styled.jsonld)
- [JSON Schema](https://stencila.dev/Styled.schema.json)
- Python class [`Styled`](https://github.com/stencila/stencila/blob/main/python/stencila/types/styled.py)
- Rust struct [`Styled`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/styled.rs)
- TypeScript class [`Styled`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Styled.ts)

## Source

This documentation was generated from [`Styled.yaml`](https://github.com/stencila/stencila/blob/main/schema/Styled.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).