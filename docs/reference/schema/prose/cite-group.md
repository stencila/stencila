---
title:
- type: Text
  value: CiteGroup
---

# Cite Group

**A group of Cite nodes.**

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

| Name  | `@id`                                                          | Type                                                               | Description                                                        | Inherited from                                                             |
| ----- | -------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | -------------------------------------------------------------------------- |
| id    | [`schema:id`](https://schema.org/id)                           | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item                                       | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)        |
| items | [`schema:itemListElement`](https://schema.org/itemListElement) | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite)*   | One or more `Cite`s to be referenced in the same surrounding text. | [`CiteGroup`](https://stencila.dev/docs/reference/schema/prose/cite-group) |

## Related

The `CiteGroup` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `CiteGroup` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `CiteGroup` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/CiteGroup.jsonld)
- [JSON Schema](https://stencila.dev/CiteGroup.schema.json)
- Python class [`CiteGroup`](https://github.com/stencila/stencila/blob/main/python/stencila/types/cite_group.py)
- Rust struct [`CiteGroup`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/cite_group.rs)
- TypeScript class [`CiteGroup`](https://github.com/stencila/stencila/blob/main/typescript/src/types/CiteGroup.ts)

## Source

This documentation was generated from [`CiteGroup.yaml`](https://github.com/stencila/stencila/blob/main/schema/CiteGroup.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).