# Cite Group

**A group of `Cite` nodes.**

This type allows you to group associated citations together.
When some content in a [`Creative Work`](./CreativeWork) cites more than one
reference for a particular piece of text, use a `CiteGroup` to encapsulate
multiple [`Cite`](./Cite) nodes.

At present we do not give a `citationMode` property to a `CiteGroup` since
they will almost always be parenthetical as opposed to narrative.
In other words, it usually only makes sense for individual `Cite` nodes to be
narrative (although they may be connected together within `content` using words
such as "and").


**`@id`**: `stencila:CiteGroup`

## Properties

The `CiteGroup` type has these properties:

| Name    | Aliases | `@id`                                                          | Type                                                                                            | Description                                                        | Inherited from                                                                                   |
| ------- | ------- | -------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `id`    | -       | [`schema:id`](https://schema.org/id)                           | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.                                      | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `items` | `item`  | [`schema:itemListElement`](https://schema.org/itemListElement) | [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)*   | One or more `Cite`s to be referenced in the same surrounding text. | -                                                                                                |

## Related

The `CiteGroup` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `CiteGroup` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `CiteGroup` type is represented in these bindings:

- [JSON-LD](https://stencila.org/CiteGroup.jsonld)
- [JSON Schema](https://stencila.org/CiteGroup.schema.json)
- Python class [`CiteGroup`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/cite_group.py)
- Rust struct [`CiteGroup`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/cite_group.rs)
- TypeScript class [`CiteGroup`](https://github.com/stencila/stencila/blob/main/ts/src/types/CiteGroup.ts)

## Source

This documentation was generated from [`CiteGroup.yaml`](https://github.com/stencila/stencila/blob/main/schema/CiteGroup.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).