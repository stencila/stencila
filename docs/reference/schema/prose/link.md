---
title:
- type: Text
  value: Link
---

# Link

**A hyperlink to other pages, sections within the same document, resources, or any URL.**

Analogues of `Link` in other schema include:
  - HTML [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)
  - JATS XML [`<ext-link>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/ext-link.html)
  - MDAST [`Link`](https://github.com/syntax-tree/mdast#link)
  - OpenDocument [`<text:a>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415212_253892949)
  - Pandoc [`Link`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L270)


**`@id`**: `stencila:Link`

## Properties

The `Link` type has these properties:

| Name    | `@id`                                                            | Type                                                                 | Description                                            | Inherited from                                                      |
| ------- | ---------------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id)                             | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The identifier for this item                           | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| content | `stencila:content`                                               | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)* | The textual content of the link.                       | [`Link`](https://stencila.dev/docs/reference/schema/prose/link)     |
| target  | `stencila:target`                                                | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The target of the link.                                | [`Link`](https://stencila.dev/docs/reference/schema/prose/link)     |
| title   | [`schema:headline`](https://schema.org/headline)                 | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | A title for the link.                                  | [`Link`](https://stencila.dev/docs/reference/schema/prose/link)     |
| rel     | [`schema:linkRelationship`](https://schema.org/linkRelationship) | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The relation between the target and the current thing. | [`Link`](https://stencila.dev/docs/reference/schema/prose/link)     |

## Related

The `Link` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Link` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                               |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a) |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                                                                     |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🔷 Low loss     |              | 🚧 Under development    | Encoded using template `[{content}]({target})`                                      |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                     |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                     |

## Bindings

The `Link` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Link.jsonld)
- [JSON Schema](https://stencila.dev/Link.schema.json)
- Python class [`Link`](https://github.com/stencila/stencila/blob/main/python/stencila/types/link.py)
- Rust struct [`Link`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/link.rs)
- TypeScript class [`Link`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Link.ts)

## Source

This documentation was generated from [`Link.yaml`](https://github.com/stencila/stencila/blob/main/schema/Link.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).