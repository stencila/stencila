---
title:
- type: Text
  value: ContactPoint
---

# Contact Point

**A contact point, usually within an organization.**

This is an implementation of schema.org [`ContactPoint`](https://schema.org/ContactPoint). It extends schema.org `ContactPoint` by, adding a `content` property which must be an array of [`Block`](./Block), as well as the properties added by [`CreativeWork`](./CreativeWork) which it extends.
`ContactPoint` is analogous, and structurally similar to, the JATS XML [`<corresp>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/corresp.html) element and the HTML5 [`<address>`](https://dev.w3.org/html5/html-author/#the-address-element) element.

**`@id`**: [`schema:ContactPoint`](https://schema.org/ContactPoint)

## Properties

The `ContactPoint` type has these properties:

| Name               | `@id`                                                              | Type                                                                                                                                                       | Description                                                                                                     | Inherited from                                                                   |
| ------------------ | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| id                 | [`schema:id`](https://schema.org/id)                               | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                                                                                    | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)              |
| alternateNames     | [`schema:alternateName`](https://schema.org/alternateName)         | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.                                                                         | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                |
| description        | [`schema:description`](https://schema.org/description)             | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                                                                                      | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                |
| identifiers        | [`schema:identifier`](https://schema.org/identifier)               | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing.                                                                   | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                |
| images             | [`schema:image`](https://schema.org/image)                         | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                                                                                             | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                |
| name               | [`schema:name`](https://schema.org/name)                           | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                                                                                           | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                |
| url                | [`schema:url`](https://schema.org/url)                             | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                                                                                            | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                |
| emails             | [`schema:email`](https://schema.org/email)                         | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Email address for correspondence.                                                                               | [`ContactPoint`](https://stencila.dev/docs/reference/schema/other/contact-point) |
| telephoneNumbers   | [`schema:telephone`](https://schema.org/telephone)                 | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Telephone numbers for the contact point.                                                                        | [`ContactPoint`](https://stencila.dev/docs/reference/schema/other/contact-point) |
| availableLanguages | [`schema:availableLanguage`](https://schema.org/availableLanguage) | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Languages (human not programming) in which it is possible to communicate with the organization/department etc.  | [`ContactPoint`](https://stencila.dev/docs/reference/schema/other/contact-point) |

## Related

The `ContactPoint` type is related to these types:

- Parents: [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)
- Children: [`PostalAddress`](https://stencila.dev/docs/reference/schema/other/postal-address)

## Formats

The `ContactPoint` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ContactPoint` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ContactPoint.jsonld)
- [JSON Schema](https://stencila.dev/ContactPoint.schema.json)
- Python class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/python/stencila/types/contact_point.py)
- Rust struct [`ContactPoint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/contact_point.rs)
- TypeScript class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ContactPoint.ts)

## Source

This documentation was generated from [`ContactPoint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ContactPoint.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).