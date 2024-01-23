# Cite

**A reference to a `CreativeWork` that is cited in another `CreativeWork`.**

A `Cite` node is used within a [`CreativeWork`](./CreativeWork), usually an
[`Article`](./Article), to refer to an other `CreativeWork`.
Often a `Cite` will be associated with other citations, in a [`CiteGroup`](./CiteGroup).


**`@id`**: `stencila:Cite`

## Properties

The `Cite` type has these properties:

| Name             | Aliases                              | `@id`                                                | Type                                                                                                                                                                                                 | Description                                                                                           | Inherited from                                                                                   |
| ---------------- | ------------------------------------ | ---------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`             | -                                    | [`schema:id`](https://schema.org/id)                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                      | The identifier for this item.                                                                         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `target`         | -                                    | [`schema:target`](https://schema.org/target)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                      | The target of the citation (URL or reference ID).                                                     | -                                                                                                |
| `citationMode`   | `citation-mode`, `citation_mode`     | `stencila:citationMode`                              | [`CitationMode`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/citation-mode.md)                                                                                        | Determines how the citation is shown within the surrounding text.                                     | -                                                                                                |
| `citationIntent` | `citation-intent`, `citation_intent` | `stencila:citationIntent`                            | [`CitationIntent`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/citation-intent.md)*                                                                                   | The type/s of the citation, both factually and rhetorically.                                          | -                                                                                                |
| `content`        | -                                    | `stencila:content`                                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)*                                                                                                    | Optional structured content/text of this citation.                                                    | -                                                                                                |
| `pageStart`      | `page-start`, `page_start`           | [`schema:pageStart`](https://schema.org/pageStart)   | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The page on which the work starts; for example "135" or "xiii".                                       | -                                                                                                |
| `pageEnd`        | `page-end`, `page_end`               | [`schema:pageEnd`](https://schema.org/pageEnd)       | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The page on which the work ends; for example "138" or "xvi".                                          | -                                                                                                |
| `pagination`     | -                                    | [`schema:pagination`](https://schema.org/pagination) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                      | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55". | -                                                                                                |
| `citationPrefix` | `citation-prefix`, `citation_prefix` | `stencila:citationPrefix`                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                      | Text to show before the citation.                                                                     | -                                                                                                |
| `citationSuffix` | `citation-suffix`, `citation_suffix` | `stencila:citationSuffix`                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                      | Text to show after the citation.                                                                      | -                                                                                                |

## Related

The `Cite` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Cite` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `Cite` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Cite.jsonld)
- [JSON Schema](https://stencila.org/Cite.schema.json)
- Python class [`Cite`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/cite.py)
- Rust struct [`Cite`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/cite.rs)
- TypeScript class [`Cite`](https://github.com/stencila/stencila/blob/main/ts/src/types/Cite.ts)

## Source

This documentation was generated from [`Cite.yaml`](https://github.com/stencila/stencila/blob/main/schema/Cite.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).