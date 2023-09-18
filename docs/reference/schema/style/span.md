# Span

**Styled inline content**

**`@id`**: `stencila:Span`

This type is marked as experimental and is likely to change.

## Properties

The `Span` type has these properties:

| Name          | `@id`                                | Type                                                                                                               | Description                                                                | Inherited from                                                                                   |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The identifier for this item                                               | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| code          | `stencila:code`                      | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                        | The code of the equation in the `styleLanguage`.                           | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| styleLanguage | `stencila:styleLanguage`             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The language used for the style specification e.g. css, tailwind, classes. | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md) | A digest of the `code` and `styleLanguage`.                                | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| errors        | `stencila:errors`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                   | Errors that occurred when transpiling the `code`.                          | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| css           | `stencila:css`                       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | A Cascading Style Sheet (CSS) transpiled from the `code` property.         | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| classes       | `stencila:classes`                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                   | A list of class names associated with the node                             | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| content       | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)*                  | The content within the span                                                | [`Span`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/span.md)     |

## Related

The `Span` type is related to these types:

- Parents: [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)
- Children: none

## Formats

The `Span` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding       | Decoding     | Status                 | Notes                                                                                     |
| --------------------------------------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         |                |              | 🚧 Under development    |                                                                                           |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟥 High loss    |              | 🚧 Under development    | Encoded using template `[{content}]{{{code}}}`                                            |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | 🟥 High loss    |              | 🟥 Alpha                |                                                                                           |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss     |              | 🟢 Stable               |                                                                                           |

## Bindings

The `Span` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Span.jsonld)
- [JSON Schema](https://stencila.dev/Span.schema.json)
- Python class [`Span`](https://github.com/stencila/stencila/blob/main/python/stencila/types/span.py)
- Rust struct [`Span`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/span.rs)
- TypeScript class [`Span`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Span.ts)

## Source

This documentation was generated from [`Span.yaml`](https://github.com/stencila/stencila/blob/main/schema/Span.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).