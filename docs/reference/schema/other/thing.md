---
title:
- type: Text
  value: Thing
---

# Thing

**The most generic type of item.**

**`@id`**: [`schema:Thing`](https://schema.org/Thing)

## Properties

The `Thing` type has these properties:

| Name           | `@id`                                                      | Type                                                                                                                                                       | Description                                   | Inherited from                                                      |
| -------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------- |
| id             | [`schema:id`](https://schema.org/id)                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                  | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| alternateNames | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.       | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| description    | [`schema:description`](https://schema.org/description)     | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                    | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| identifiers    | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing. | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| images         | [`schema:image`](https://schema.org/image)                 | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                           | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| name           | [`schema:name`](https://schema.org/name)                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                         | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| url            | [`schema:url`](https://schema.org/url)                     | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                          | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |

## Related

The `Thing` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: [`Brand`](https://stencila.dev/docs/reference/schema/other/brand), [`ContactPoint`](https://stencila.dev/docs/reference/schema/other/contact-point), [`CreativeWork`](https://stencila.dev/docs/reference/schema/works/creative-work), [`DatatableColumn`](https://stencila.dev/docs/reference/schema/data/datatable-column), [`DefinedTerm`](https://stencila.dev/docs/reference/schema/prose/defined-term), [`Enumeration`](https://stencila.dev/docs/reference/schema/other/enumeration), [`Grant`](https://stencila.dev/docs/reference/schema/other/grant), [`ListItem`](https://stencila.dev/docs/reference/schema/prose/list-item), [`Organization`](https://stencila.dev/docs/reference/schema/other/organization), [`Person`](https://stencila.dev/docs/reference/schema/other/person), [`Product`](https://stencila.dev/docs/reference/schema/other/product), [`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value)

## Formats

The `Thing` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `Thing` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Thing.jsonld)
- [JSON Schema](https://stencila.dev/Thing.schema.json)
- Python class [`Thing`](https://github.com/stencila/stencila/blob/main/python/stencila/types/thing.py)
- Rust struct [`Thing`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/thing.rs)
- TypeScript class [`Thing`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Thing.ts)

## Source

This documentation was generated from [`Thing.yaml`](https://github.com/stencila/stencila/blob/main/schema/Thing.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).