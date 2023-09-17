---
title:
- type: Text
  value: ThematicBreak
---

# Thematic Break

**A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
**

Analogues of `ThematicBreak` in other schema include:
  - JATS XML [`<hr>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/hr.html)
  - MDAST [`ThematicBreak`](https://github.com/syntax-tree/mdast#ThematicBreak)
  - OpenDocument OpenDocument [`<text:soft-page-break>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#element-text_soft-page-break)


**`@id`**: `stencila:ThematicBreak`

## Properties

The `ThematicBreak` type has these properties:

| Name | `@id`                                | Type                                                               | Description                  | Inherited from                                                      |
| ---- | ------------------------------------ | ------------------------------------------------------------------ | ---------------------------- | ------------------------------------------------------------------- |
| id   | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |

## Related

The `ThematicBreak` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `ThematicBreak` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                            |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | -------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag `<hr>`            |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟢 No loss      |              | 🚧 Under development    | Encoded using template `---\n\n` |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                  |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                  |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                  |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                  |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                  |

## Bindings

The `ThematicBreak` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ThematicBreak.jsonld)
- [JSON Schema](https://stencila.dev/ThematicBreak.schema.json)
- Python class [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/python/stencila/types/thematic_break.py)
- Rust struct [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/thematic_break.rs)
- TypeScript class [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ThematicBreak.ts)

## Source

This documentation was generated from [`ThematicBreak.yaml`](https://github.com/stencila/stencila/blob/main/schema/ThematicBreak.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).