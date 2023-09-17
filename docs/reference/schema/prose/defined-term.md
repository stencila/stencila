---
title:
- type: Text
  value: DefinedTerm
---

# Defined Term

**A word, name, acronym, phrase, etc. with a formal definition.**

Often used in the context of category or subject classification,  glossaries or dictionaries, product or creative work types, etc.
Use the `name` property for the term being defined, use `termCode`. If the term has an alpha-numeric code allocated, use
description to provide the definition of the term.


**`@id`**: [`schema:DefinedTerm`](https://schema.org/DefinedTerm)

## Properties

The `DefinedTerm` type has these properties:

| Name           | `@id`                                                      | Type                                                                                                                                                       | Description                                                     | Inherited from                                                                 |
| -------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| id             | [`schema:id`](https://schema.org/id)                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                                    | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)            |
| alternateNames | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.                         | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)              |
| description    | [`schema:description`](https://schema.org/description)     | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                                      | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)              |
| identifiers    | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing.                   | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)              |
| images         | [`schema:image`](https://schema.org/image)                 | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                                             | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)              |
| name           | [`schema:name`](https://schema.org/name)                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                                           | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)              |
| url            | [`schema:url`](https://schema.org/url)                     | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                                            | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)              |
| termCode       | [`schema:termCode`](https://schema.org/termCode)           | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | A code that identifies this DefinedTerm within a DefinedTermSet | [`DefinedTerm`](https://stencila.dev/docs/reference/schema/prose/defined-term) |

## Related

The `DefinedTerm` type is related to these types:

- Parents: [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)
- Children: none

## Formats

The `DefinedTerm` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `DefinedTerm` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/DefinedTerm.jsonld)
- [JSON Schema](https://stencila.dev/DefinedTerm.schema.json)
- Python class [`DefinedTerm`](https://github.com/stencila/stencila/blob/main/python/stencila/types/defined_term.py)
- Rust struct [`DefinedTerm`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/defined_term.rs)
- TypeScript class [`DefinedTerm`](https://github.com/stencila/stencila/blob/main/typescript/src/types/DefinedTerm.ts)

## Source

This documentation was generated from [`DefinedTerm.yaml`](https://github.com/stencila/stencila/blob/main/schema/DefinedTerm.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).