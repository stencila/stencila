---
title:
- type: Text
  value: PropertyValue
---

# Property Value

**A property-value pair.**

Always use specific properties when they exist and you can populate them.
Using `PropertyValue` as a substitute will typically not convey as much semantic
information as using the specific property.

Most of the time a `PropertyValue` node will need a `value` property
(e.g. most of the examples on https://schema.org/PropertyValue have one)
so this schema make that property required.

This type is mainly provided for use in `Thing.identifiers` (see the notes there).


**`@id`**: [`schema:PropertyValue`](https://schema.org/PropertyValue)

## Properties

The `PropertyValue` type has these properties:

| Name           | `@id`                                                      | Type                                                                                                                                                       | Description                                                                    | Inherited from                                                                     |
| -------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------- |
| id             | [`schema:id`](https://schema.org/id)                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                                                   | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                |
| alternateNames | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.                                        | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| description    | [`schema:description`](https://schema.org/description)     | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                                                     | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| identifiers    | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing.                                  | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| images         | [`schema:image`](https://schema.org/image)                 | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                                                            | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| name           | [`schema:name`](https://schema.org/name)                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                                                          | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| url            | [`schema:url`](https://schema.org/url)                     | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                                                           | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| propertyID     | [`schema:propertyID`](https://schema.org/propertyID)       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | A commonly used identifier for the characteristic represented by the property. | [`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) |
| value          | [`schema:value`](https://schema.org/value)                 | [`Primitive`](https://stencila.dev/docs/reference/schema/data/primitive)                                                                                   | The value of the property.                                                     | [`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) |

## Related

The `PropertyValue` type is related to these types:

- Parents: [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)
- Children: none

## Formats

The `PropertyValue` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `PropertyValue` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/PropertyValue.jsonld)
- [JSON Schema](https://stencila.dev/PropertyValue.schema.json)
- Python class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/python/stencila/types/property_value.py)
- Rust struct [`PropertyValue`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/property_value.rs)
- TypeScript class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/typescript/src/types/PropertyValue.ts)

## Source

This documentation was generated from [`PropertyValue.yaml`](https://github.com/stencila/stencila/blob/main/schema/PropertyValue.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).