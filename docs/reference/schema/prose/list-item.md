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

| Name             | Aliases                                                                                   | `@id`                                                      | Type                                                                                                                                                                                                                 | Description                                                | Inherited from                                                                                   |
| ---------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`             | -                                                                                         | [`schema:id`](https://schema.org/id)                       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                      | The identifier for this item.                              | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `alternateNames` | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                     | Alternate names (aliases) for the item.                    | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `description`    | -                                                                                         | [`schema:description`](https://schema.org/description)     | [`Text`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md)                                                                                                                         | A description of the item.                                 | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `identifiers`    | `identifier`                                                                              | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property-value.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md))* | Any kind of identifier for any kind of Thing.              | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `images`         | `image`                                                                                   | [`schema:image`](https://schema.org/image)                 | [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)*                                                                                                         | Images of the item.                                        | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `name`           | -                                                                                         | [`schema:name`](https://schema.org/name)                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                      | The name of the item.                                      | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `url`            | -                                                                                         | [`schema:url`](https://schema.org/url)                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                      | The URL of the item.                                       | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `content`        | -                                                                                         | `stencila:content`                                         | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                                                                                                                      | The content of the list item.                              | -                                                                                                |
| `item`           | -                                                                                         | [`schema:item`](https://schema.org/item)                   | [`Node`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/node.md)                                                                                                                         | The item represented by this list item.                    | -                                                                                                |
| `isChecked`      | `is-checked`, `is_checked`                                                                | `stencila:isChecked`                                       | [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)                                                                                                                    | A flag to indicate if this list item is checked.           | -                                                                                                |
| `position`       | -                                                                                         | [`schema:position`](https://schema.org/position)           | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                                                                                                                    | The position of the item in a series or sequence of items. | -                                                                                                |

## Related

The `ListItem` type is related to these types:

- Parents: [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)
- Children: none

## Formats

The `ListItem` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding      | Status                 | Notes                                                                                                        |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------- | ---------------------- | ------------------------------------------------------------------------------------------------------------ |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |               | 🚧 Under development    | Encoded as [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)                            |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              | 🔷 Low loss       |               | 🚧 Under development    | Encoded as [`<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list-item.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | 🔷 Low loss       | 🔷 Low loss    | ⚠️ Alpha               | Encoded using special function                                                                               |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |               | ⚠️ Alpha               |                                                                                                              |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                                              |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                                              |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss     | 🔶 Beta                 |                                                                                                              |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                                              |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                                              |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                                              |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |               | 🟢 Stable               |                                                                                                              |

## Bindings

The `ListItem` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ListItem.jsonld)
- [JSON Schema](https://stencila.org/ListItem.schema.json)
- Python class [`ListItem`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/list_item.py)
- Rust struct [`ListItem`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list_item.rs)
- TypeScript class [`ListItem`](https://github.com/stencila/stencila/blob/main/ts/src/types/ListItem.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `ListItem` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                     | Strategy                  |
| --------- | ---------- | ----------------------------------------------- | ------------------------- |
| `content` | Min+       | Generate a single, arbitrary, paragraph         | `vec_paragraphs(1)`       |
|           | Low+       | Generate one, arbitrary, non-list block         | `vec_blocks_list_item(1)` |
|           | High+      | Generate up to two, arbitrary, non-list blocks  | `vec_blocks_list_item(2)` |
|           | Max        | Generate up to four, arbitrary, non-list blocks | `vec_blocks_list_item(4)` |

## Source

This documentation was generated from [`ListItem.yaml`](https://github.com/stencila/stencila/blob/main/schema/ListItem.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.