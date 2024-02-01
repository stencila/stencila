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

| Name             | Aliases                                                                                   | `@id`                                                      | Type                                                                                                                                                                                                                  | Description                                                                    | Inherited from                                                                                   |
| ---------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `id`             | -                                                                                         | [`schema:id`](https://schema.org/id)                       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The identifier for this item.                                                  | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `alternateNames` | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                      | Alternate names (aliases) for the item.                                        | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `description`    | -                                                                                         | [`schema:description`](https://schema.org/description)     | [`Text`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md)                                                                                                                          | A description of the item.                                                     | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `identifiers`    | `identifier`                                                                              | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property-value.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md))* | Any kind of identifier for any kind of Thing.                                  | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `images`         | `image`                                                                                   | [`schema:image`](https://schema.org/image)                 | [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)*                                                                                                          | Images of the item.                                                            | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `name`           | -                                                                                         | [`schema:name`](https://schema.org/name)                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The name of the item.                                                          | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `url`            | -                                                                                         | [`schema:url`](https://schema.org/url)                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The URL of the item.                                                           | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `propertyID`     | `property-id`, `property_id`                                                              | [`schema:propertyID`](https://schema.org/propertyID)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | A commonly used identifier for the characteristic represented by the property. | -                                                                                                |
| `value`          | -                                                                                         | [`schema:value`](https://schema.org/value)                 | [`Primitive`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/primitive.md)                                                                                                                 | The value of the property.                                                     | -                                                                                                |

## Related

The `PropertyValue` type is related to these types:

- Parents: [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)
- Children: none

## Formats

The `PropertyValue` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.md)           | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)    |              |           | 🚧 Under development |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `PropertyValue` type is represented in these bindings:

- [JSON-LD](https://stencila.org/PropertyValue.jsonld)
- [JSON Schema](https://stencila.org/PropertyValue.schema.json)
- Python class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/property_value.py)
- Rust struct [`PropertyValue`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/property_value.rs)
- TypeScript class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/ts/src/types/PropertyValue.ts)

## Source

This documentation was generated from [`PropertyValue.yaml`](https://github.com/stencila/stencila/blob/main/schema/PropertyValue.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).