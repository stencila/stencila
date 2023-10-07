# Division

**Styled block content**

**`@id`**: `stencila:Division`

This type is marked as unstable and is subject to change.

## Properties

The `Division` type has these properties:

| Name          | `@id`                                | Type                                                                                                               | Description                                                                | Inherited from                                                                                       |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The identifier for this item                                               | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)     |
| code          | `stencila:code`                      | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                        | The code of the equation in the `styleLanguage`.                           | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)     |
| styleLanguage | `stencila:styleLanguage`             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The language used for the style specification e.g. css, tailwind, classes. | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)     |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md) | A digest of the `code` and `styleLanguage`.                                | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)     |
| errors        | `stencila:errors`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                   | Errors that occurred when transpiling the `code`.                          | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)     |
| css           | `stencila:css`                       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | A Cascading Style Sheet (CSS) transpiled from the `code` property.         | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)     |
| classes       | `stencila:classes`                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                   | A list of class names associated with the node                             | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)     |
| content       | `stencila:content`                   | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                    | The content within the division                                            | [`Division`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/division.md) |

## Related

The `Division` type is related to these types:

- Parents: [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)
- Children: none

## Formats

The `Division` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                                   |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | --------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    | Encoded to tag [`<div>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         |                  |              | 🚧 Under development    |                                                                                         |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | ⚠️ High loss     |              | 🚧 Under development    | Encoded using special function                                                          |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                         |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                         |

## Bindings

The `Division` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Division.jsonld)
- [JSON Schema](https://stencila.dev/Division.schema.json)
- Python class [`Division`](https://github.com/stencila/stencila/blob/main/python/stencila/types/division.py)
- Rust struct [`Division`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/division.rs)
- TypeScript class [`Division`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Division.ts)

## Source

This documentation was generated from [`Division.yaml`](https://github.com/stencila/stencila/blob/main/schema/Division.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).