# Delete

**A suggestion to delete some inline content.**

**`@id`**: `stencila:Delete`

## Properties

The `Delete` type has these properties:

| Name    | `@id`                                | Type                                                                                              | Description                                              | Inherited from                                                                                           |
| ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item                             | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)         |
| content | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The content that is suggested to be inserted or deleted. | [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/suggestion.md) |

## Related

The `Delete` type is related to these types:

- Parents: [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/suggestion.md)
- Children: none

## Formats

The `Delete` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                                   |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | --------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    | Encoded to tag [`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         |                  |              | 🚧 Under development    | Encoded using special function                                                          |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | ⚠️ High loss     |              | 🚧 Under development    | Encoded using template `<del>{content}</del>`                                           |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                         |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                         |

## Bindings

The `Delete` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Delete.jsonld)
- [JSON Schema](https://stencila.dev/Delete.schema.json)
- Python class [`Delete`](https://github.com/stencila/stencila/blob/main/python/stencila/types/delete.py)
- Rust struct [`Delete`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/delete.rs)
- TypeScript class [`Delete`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Delete.ts)

## Source

This documentation was generated from [`Delete.yaml`](https://github.com/stencila/stencila/blob/main/schema/Delete.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).