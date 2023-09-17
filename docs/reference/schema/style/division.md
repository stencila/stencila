---
title:
- type: Text
  value: Division
---

# Division

**Styled block content**

**`@id`**: `stencila:Division`

This type is marked as experimental and is likely to change.

## Properties

The `Division` type has these properties:

| Name          | `@id`                                | Type                                                                                  | Description                                                                | Inherited from                                                          |
| ------------- | ------------------------------------ | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| id            | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The identifier for this item                                               | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)     |
| code          | `stencila:code`                      | [`Cord`](https://stencila.dev/docs/reference/schema/data/cord)                        | The code of the equation in the `styleLanguage`.                           | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled)     |
| styleLanguage | `stencila:styleLanguage`             | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | The language used for the style specification e.g. css, tailwind, classes. | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled)     |
| compileDigest | `stencila:compileDigest`             | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) | A digest of the `code` and `styleLanguage`.                                | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled)     |
| errors        | `stencila:errors`                    | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                   | Errors that occurred when transpiling the `code`.                          | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled)     |
| css           | `stencila:css`                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                    | A Cascading Style Sheet (CSS) transpiled from the `code` property.         | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled)     |
| classes       | `stencila:classes`                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                   | A list of class names associated with the node                             | [`Styled`](https://stencila.dev/docs/reference/schema/style/styled)     |
| content       | `stencila:content`                   | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                    | The content within the division                                            | [`Division`](https://stencila.dev/docs/reference/schema/style/division) |

## Related

The `Division` type is related to these types:

- Parents: [`Styled`](https://stencila.dev/docs/reference/schema/style/styled)
- Children: none

## Formats

The `Division` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                   |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | --------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<div>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div) |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                                                                         |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    | Encoded using template `::: {{{code}}}\n\n{content}:::\n\n`                             |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                         |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                         |

## Bindings

The `Division` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Division.jsonld)
- [JSON Schema](https://stencila.dev/Division.schema.json)
- Python class [`Division`](https://github.com/stencila/stencila/blob/main/python/stencila/types/division.py)
- Rust struct [`Division`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/division.rs)
- TypeScript class [`Division`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Division.ts)

## Source

This documentation was generated from [`Division.yaml`](https://github.com/stencila/stencila/blob/main/schema/Division.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).