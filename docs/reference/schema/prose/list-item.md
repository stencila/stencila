---
title:
- type: Text
  value: ListItem
---

# List Item

**A single item in a list.**

This is an implementation, and extension, of schema.org [`ListItem`](https://schema.org/ListItem).
It extends schema.ord `ListItem` by adding `content` and `isChecked` properties.

Analogues of `ListItem` in other schema include:
  - JATS XML `<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/list-item.html)
  - HTML [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)
  - MDAST [`ListItem`](https://github.com/syntax-tree/mdast#listitem)
  - OpenDocument [`<text:list-item>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415154_253892949)


**`@id`**: [`schema:ListItem`](https://schema.org/ListItem)

## Properties

The `ListItem` type has these properties:

| Name           | `@id`                                                      | Type                                                                                                                                                       | Description                                                | Inherited from                                                           |
| -------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------ |
| id             | [`schema:id`](https://schema.org/id)                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                               | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)      |
| alternateNames | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.                    | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)        |
| description    | [`schema:description`](https://schema.org/description)     | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                                 | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)        |
| identifiers    | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing.              | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)        |
| images         | [`schema:image`](https://schema.org/image)                 | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                                        | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)        |
| name           | [`schema:name`](https://schema.org/name)                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                                      | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)        |
| url            | [`schema:url`](https://schema.org/url)                     | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                                       | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)        |
| content        | `stencila:content`                                         | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)* \| [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)*                 | The content of the list item.                              | [`ListItem`](https://stencila.dev/docs/reference/schema/prose/list-item) |
| item           | [`schema:item`](https://schema.org/item)                   | [`Node`](https://stencila.dev/docs/reference/schema/other/node)                                                                                            | The item represented by this list item.                    | [`ListItem`](https://stencila.dev/docs/reference/schema/prose/list-item) |
| isChecked      | `stencila:isChecked`                                       | [`Boolean`](https://stencila.dev/docs/reference/schema/data/boolean)                                                                                       | A flag to indicate if this list item is checked.           | [`ListItem`](https://stencila.dev/docs/reference/schema/prose/list-item) |
| position       | [`schema:position`](https://schema.org/position)           | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)                                                                                       | The position of the item in a series or sequence of items. | [`ListItem`](https://stencila.dev/docs/reference/schema/prose/list-item) |

## Related

The `ListItem` type is related to these types:

- Parents: [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)
- Children: none

## Formats

The `ListItem` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                                       |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)                       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list-item) |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🔷 Low loss     |              | 🚧 Under development    | Encoded using special function                                                                              |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                                             |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                             |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                             |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                             |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                                             |

## Bindings

The `ListItem` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ListItem.jsonld)
- [JSON Schema](https://stencila.dev/ListItem.schema.json)
- Python class [`ListItem`](https://github.com/stencila/stencila/blob/main/python/stencila/types/list_item.py)
- Rust struct [`ListItem`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list_item.rs)
- TypeScript class [`ListItem`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ListItem.ts)

## Source

This documentation was generated from [`ListItem.yaml`](https://github.com/stencila/stencila/blob/main/schema/ListItem.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).