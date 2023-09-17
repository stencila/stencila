---
title:
- type: Text
  value: Cite
---

# Cite

**A reference to a CreativeWork that is cited in another CreativeWork.**

A `Cite` node is used within a [`CreativeWork`](./CreativeWork), usually an
[`Article`](./Article), to refer to an other `CreativeWork`.
Often a `Cite` will be associated with other citations, in a [`CiteGroup`](./CiteGroup).


**`@id`**: `stencila:Cite`

## Properties

The `Cite` type has these properties:

| Name           | `@id`                                                | Type                                                                                                                                      | Description                                                                                            | Inherited from                                                      |
| -------------- | ---------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------- |
| id             | [`schema:id`](https://schema.org/id)                 | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                        | The identifier for this item                                                                           | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| target         | `stencila:target`                                    | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                        | The target of the citation (URL or reference ID).                                                      | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)     |
| citationMode   | `stencila:citationMode`                              | [`CitationMode`](https://stencila.dev/docs/reference/schema/prose/citation-mode)                                                          | Determines how the citation is shown within the surrounding text.                                      | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)     |
| citationIntent | `stencila:citationIntent`                            | [`CitationIntent`](https://stencila.dev/docs/reference/schema/prose/citation-intent)*                                                     | The type/s of the citation, both factually and rhetorically.                                           | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)     |
| content        | `stencila:content`                                   | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)*                                                                      | Optional structured content/text of this citation.                                                     | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)     |
| pageStart      | [`schema:pageStart`](https://schema.org/pageStart)   | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer) \| [`String`](https://stencila.dev/docs/reference/schema/data/string) | The page on which the work starts; for example "135" or "xiii".                                        | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)     |
| pageEnd        | [`schema:pageEnd`](https://schema.org/pageEnd)       | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer) \| [`String`](https://stencila.dev/docs/reference/schema/data/string) | The page on which the work ends; for example "138" or "xvi".                                           | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)     |
| pagination     | [`schema:pagination`](https://schema.org/pagination) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                        | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".  | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)     |
| citationPrefix | `stencila:citationPrefix`                            | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                        | Text to show before the citation.                                                                      | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)     |
| citationSuffix | `stencila:citationSuffix`                            | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                        | Text to show after the citation.                                                                       | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)     |

## Related

The `Cite` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Cite` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Cite` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Cite.jsonld)
- [JSON Schema](https://stencila.dev/Cite.schema.json)
- Python class [`Cite`](https://github.com/stencila/stencila/blob/main/python/stencila/types/cite.py)
- Rust struct [`Cite`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/cite.rs)
- TypeScript class [`Cite`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Cite.ts)

## Source

This documentation was generated from [`Cite.yaml`](https://github.com/stencila/stencila/blob/main/schema/Cite.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).