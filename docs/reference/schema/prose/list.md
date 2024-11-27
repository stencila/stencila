# List

**A list of items.**

This is an implementation, and renaming, of schema.org [`ItemList`](https://schema.org/ItemList).
Renaming was done as `List` was considered a more developer friendly alternative. Similarly,
schema.org properties `itemListElement` and `itemListOrder` were renamed to `items` and `order`.
Note that, as with every other such renaming in Stencila Schema, a mapping between names is
defined and it is trivial to save Stencila Schema documents using the schema.org vocabulary if so desired.


**`@id`**: [`schema:ItemList`](https://schema.org/ItemList)

## Properties

The `List` type has these properties:

| Name         | Aliases  | `@id`                                                          | Type                                                                                                                 | Description                                                 | Inherited from                                                                                   |
| ------------ | -------- | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`         | -        | [`schema:id`](https://schema.org/id)                           | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                      | The identifier for this item.                               | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `items`      | `item`   | [`schema:itemListElement`](https://schema.org/itemListElement) | [`ListItem`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list-item.md)*               | The items in the list.                                      | -                                                                                                |
| `order`      | -        | [`schema:itemListOrder`](https://schema.org/itemListOrder)     | [`ListOrder`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list-order.md)              | The ordering of the list.                                   | -                                                                                                |
| `authors`    | `author` | [`schema:author`](https://schema.org/author)                   | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                    | The authors of the list.                                    | -                                                                                                |
| `provenance` | -        | `stencila:provenance`                                          | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)* | A summary of the provenance of the content within the list. | -                                                                                                |

## Related

The `List` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `List` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                                                                                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | -------------------------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |                                                                                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development | Encoded using special function                                                                     |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🔷 Low loss   |            | 🚧 Under development | Encoded as [`<list>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              | Encoded using implemented function                                                                 |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              |                                                                                                    |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              |                                                                                                    |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              |                                                                                                    |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              |                                                                                                    |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                    |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |                                                                                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |                                                                                                    |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                    |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                    |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |                                                                                                    |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |                                                                                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |                                                                                                    |

## Bindings

The `List` type is represented in these bindings:

- [JSON-LD](https://stencila.org/List.jsonld)
- [JSON Schema](https://stencila.org/List.schema.json)
- Python class [`List`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/list.py)
- Rust struct [`List`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list.rs)
- TypeScript class [`List`](https://github.com/stencila/stencila/blob/main/ts/src/types/List.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `List` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                               | Strategy                                                             |
| -------- | ---------- | --------------------------------------------------------- | -------------------------------------------------------------------- |
| `items`  | Min+       | Generate a single, arbitrary, list item.                  | `vec(ListItem::arbitrary(), size_range(1..=1))`                      |
|          | Low+       | Generate up to two, arbitrary, list items.                | `vec(ListItem::arbitrary(), size_range(1..=2))`                      |
|          | High+      | Generate up to four, arbitrary, list items.               | `vec(ListItem::arbitrary(), size_range(1..=4))`                      |
|          | Max        | Generate up to eight, arbitrary, list items.              | `vec(ListItem::arbitrary(), size_range(1..=8))`                      |
| `order`  | Min+       | Always generate an unordered list.                        | `ListOrder::Unordered`                                               |
|          | Low+       | Randomly generate either and unordered or ascending list. | `prop_oneof![Just(ListOrder::Unordered),Just(ListOrder::Ascending)]` |
|          | High+      | Generate an arbitrary list ordering.                      | `ListOrder::arbitrary()`                                             |

## Source

This documentation was generated from [`List.yaml`](https://github.com/stencila/stencila/blob/main/schema/List.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
