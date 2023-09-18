# Cite

**A reference to a CreativeWork that is cited in another CreativeWork.**

A `Cite` node is used within a [`CreativeWork`](./CreativeWork), usually an
[`Article`](./Article), to refer to an other `CreativeWork`.
Often a `Cite` will be associated with other citations, in a [`CiteGroup`](./CiteGroup).


**`@id`**: `stencila:Cite`

## Properties

The `Cite` type has these properties:

| Name           | `@id`                                                | Type                                                                                                                                                                                                | Description                                                                                            | Inherited from                                                                                   |
| -------------- | ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| id             | [`schema:id`](https://schema.org/id)                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                     | The identifier for this item                                                                           | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| target         | `stencila:target`                                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                     | The target of the citation (URL or reference ID).                                                      | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)     |
| citationMode   | `stencila:citationMode`                              | [`CitationMode`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/citation-mode.md)                                                                                       | Determines how the citation is shown within the surrounding text.                                      | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)     |
| citationIntent | `stencila:citationIntent`                            | [`CitationIntent`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/citation-intent.md)*                                                                                  | The type/s of the citation, both factually and rhetorically.                                           | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)     |
| content        | `stencila:content`                                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)*                                                                                                   | Optional structured content/text of this citation.                                                     | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)     |
| pageStart      | [`schema:pageStart`](https://schema.org/pageStart)   | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The page on which the work starts; for example "135" or "xiii".                                        | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)     |
| pageEnd        | [`schema:pageEnd`](https://schema.org/pageEnd)       | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The page on which the work ends; for example "138" or "xvi".                                           | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)     |
| pagination     | [`schema:pagination`](https://schema.org/pagination) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                     | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".  | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)     |
| citationPrefix | `stencila:citationPrefix`                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                     | Text to show before the citation.                                                                      | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)     |
| citationSuffix | `stencila:citationSuffix`                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                     | Text to show after the citation.                                                                       | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)     |

## Related

The `Cite` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Cite` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding       | Decoding     | Status                 | Notes |
| --------------------------------------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         |                |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Cite` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Cite.jsonld)
- [JSON Schema](https://stencila.dev/Cite.schema.json)
- Python class [`Cite`](https://github.com/stencila/stencila/blob/main/python/stencila/types/cite.py)
- Rust struct [`Cite`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/cite.rs)
- TypeScript class [`Cite`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Cite.ts)

## Source

This documentation was generated from [`Cite.yaml`](https://github.com/stencila/stencila/blob/main/schema/Cite.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).