---
title:
- type: Text
  value: Styled
---

# Styled

**An abstract base class for a document node that has styling applied to it and/or its content**

This class is very similar to the `Math` abstract base class but has `styleLanguage` instead
of `mathLanguage` and compiled `css` instead of `mathml`.

Note also that `styleLanguage` is optional.


**`@id`**: `stencila:Styled`

This type is marked as experimental and is likely to change.

## Properties

The `Styled` type has these properties:

| Name          | `@id`                                | Type                                                                                  | Description                                                                | Inherited from                                                      |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The identifier for this item                                               | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| code          | `stencila:code`                      | [`Cord`](https://stencila.dev/docs/reference/schema/data/cord)                        | The code of the equation in the `styleLanguage`.                           | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| styleLanguage | `stencila:styleLanguage`             | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The language used for the style specification e.g. css, tailwind, classes. | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) | A digest of the `code` and `styleLanguage`.                                | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| errors        | `stencila:errors`                    | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                   | Errors that occurred when transpiling the `code`.                          | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| css           | `stencila:css`                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | A Cascading Style Sheet (CSS) transpiled from the `code` property.         | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| classes       | `stencila:classes`                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                   | A list of class names associated with the node                             | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |

## Related

The `Styled` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: [`Division`](https://stencila.dev/docs/reference/schema/style/division), [`Span`](https://stencila.dev/docs/reference/schema/style/span)

## Bindings

The `Styled` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Styled.jsonld)
- [JSON Schema](https://stencila.dev/Styled.schema.json)
- Python class [`Styled`](https://github.com/stencila/stencila/blob/main/python/stencila/types/styled.py)
- Rust struct [`Styled`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/styled.rs)
- TypeScript class [`Styled`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Styled.ts)

## Source

This documentation was generated from [`Styled.yaml`](https://github.com/stencila/stencila/blob/main/schema/Styled.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).