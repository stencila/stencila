---
title:
- type: Text
  value: Brand
---

# Brand

**A brand used by an organization or person for labeling a product, product group, or similar.**

**`@id`**: [`schema:Brand`](https://schema.org/Brand)

## Properties

The `Brand` type has these properties:

| Name           | `@id`                                                      | Type                                                                                                                                                       | Description                                   | Inherited from                                                      |
| -------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------- |
| id             | [`schema:id`](https://schema.org/id)                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                  | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| alternateNames | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.       | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| description    | [`schema:description`](https://schema.org/description)     | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                    | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| identifiers    | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing. | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| images         | [`schema:image`](https://schema.org/image)                 | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                           | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| name           | [`schema:name`](https://schema.org/name)                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                         | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| url            | [`schema:url`](https://schema.org/url)                     | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                          | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| logo           | [`schema:logo`](https://schema.org/logo)                   | [`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string)       | A logo associated with the brand.             | [`Brand`](https://stencila.dev/docs/reference/schema/other/brand)   |
| reviews        | [`schema:review`](https://schema.org/review)               | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Reviews of the brand.                         | [`Brand`](https://stencila.dev/docs/reference/schema/other/brand)   |

## Related

The `Brand` type is related to these types:

- Parents: [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)
- Children: none

## Formats

The `Brand` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Brand` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Brand.jsonld)
- [JSON Schema](https://stencila.dev/Brand.schema.json)
- Python class [`Brand`](https://github.com/stencila/stencila/blob/main/python/stencila/types/brand.py)
- Rust struct [`Brand`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/brand.rs)
- TypeScript class [`Brand`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Brand.ts)

## Source

This documentation was generated from [`Brand.yaml`](https://github.com/stencila/stencila/blob/main/schema/Brand.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).