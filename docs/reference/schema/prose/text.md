# Text

**Textual content**

Intended mostly for use for inline text e.g. the text in a paragraph.

Differs from the primitive `String` type in that it has a `type` and `id` property.
The `id` property allows use to identify text nodes with a sequence of inline nodes
for better diffing.

Also, in Rust, the `value` property is implemented as a CRDT.


**`@id`**: [`schema:Text`](https://schema.org/Text)

## Properties

The `Text` type has these properties:

| Name  | `@id`                                      | Type                                                                                            | Description                   | Inherited from                                                                                   |
| ----- | ------------------------------------------ | ----------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| id    | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item  | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| value | [`schema:value`](https://schema.org/value) | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)     | The value of the text content | [`Text`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md)     |

## Related

The `Text` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Text` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding      | Decoding     | Status                 | Notes                                                                                     |
| --------------------------------------------------------------------------------------------- | ------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🟢 No loss     |              | 🚧 Under development    | Encoded to tag [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🟢 No loss     |              | 🚧 Under development    | Encoded using special function                                                            |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟢 No loss     |              | 🚧 Under development    |                                                                                           |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | 🟢 No loss     |              | 🟥 Alpha                |                                                                                           |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss     | 🟢 No loss    | 🟢 Stable               |                                                                                           |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss    |              | 🟢 Stable               |                                                                                           |

## Bindings

The `Text` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Text.jsonld)
- [JSON Schema](https://stencila.dev/Text.schema.json)
- Python class [`Text`](https://github.com/stencila/stencila/blob/main/python/stencila/types/text.py)
- Rust struct [`Text`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/text.rs)
- TypeScript class [`Text`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Text.ts)

## Source

This documentation was generated from [`Text.yaml`](https://github.com/stencila/stencila/blob/main/schema/Text.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).