---
title:
- type: Text
  value: Span
---

# Span

**Styled inline content**

**`@id`**: `stencila:Span`

This type is marked as experimental and is likely to change.

## Properties

The `Span` type has these properties:

| Name          | `@id`                                | Type                                                                                  | Description                                                                | Inherited from                                                      |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The identifier for this item                                               | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| code          | `stencila:code`                      | [`Cord`](https://stencila.dev/docs/reference/schema/data/cord)                        | The code of the equation in the `styleLanguage`.                           | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| styleLanguage | `stencila:styleLanguage`             | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The language used for the style specification e.g. css, tailwind, classes. | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) | A digest of the `code` and `styleLanguage`.                                | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| errors        | `stencila:errors`                    | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                   | Errors that occurred when transpiling the `code`.                          | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| css           | `stencila:css`                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | A Cascading Style Sheet (CSS) transpiled from the `code` property.         | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| classes       | `stencila:classes`                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                   | A list of class names associated with the node                             | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled) |
| content       | `stencila:content`                   | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)*                  | The content within the span                                                | [`Span`](https://stencila.dev/docs/reference/schema/style/span)     |

## Related

The `Span` type is related to these types:

- Parents: [`Styled`](https://stencila.dev/docs/reference/schema/style/styled)
- Children: none

## Formats

The `Span` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                     |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span) |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                                                                           |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    | Encoded using template `[{content}]{{{code}}}`                                            |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                           |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                           |

## Bindings

The `Span` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Span.jsonld)
- [JSON Schema](https://stencila.dev/Span.schema.json)
- Python class [`Span`](https://github.com/stencila/stencila/blob/main/python/stencila/types/span.py)
- Rust struct [`Span`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/span.rs)
- TypeScript class [`Span`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Span.ts)

## Source

This documentation was generated from [`Span.yaml`](https://github.com/stencila/stencila/blob/main/schema/Span.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).