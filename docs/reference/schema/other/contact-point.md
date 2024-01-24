# Contact Point

**A contact point, usually within an organization.**

This is an implementation of schema.org [`ContactPoint`](https://schema.org/ContactPoint). It extends schema.org `ContactPoint` by, adding a `content` property which must be an array of [`Block`](./Block), as well as the properties added by [`CreativeWork`](./CreativeWork) which it extends.
`ContactPoint` is analogous, and structurally similar to, the JATS XML [`<corresp>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/corresp.html) element and the HTML5 [`<address>`](https://dev.w3.org/html5/html-author/#the-address-element) element.

**`@id`**: [`schema:ContactPoint`](https://schema.org/ContactPoint)

## Properties

The `ContactPoint` type has these properties:

| Name                 | Aliases                                                                                                          | `@id`                                                              | Type                                                                                                                                                                                                                  | Description                                                                                                    | Inherited from                                                                                   |
| -------------------- | ---------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                 | -                                                                                                                | [`schema:id`](https://schema.org/id)                               | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The identifier for this item.                                                                                  | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `alternateNames`     | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name`                        | [`schema:alternateName`](https://schema.org/alternateName)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                      | Alternate names (aliases) for the item.                                                                        | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `description`        | -                                                                                                                | [`schema:description`](https://schema.org/description)             | [`Text`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md)                                                                                                                          | A description of the item.                                                                                     | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `identifiers`        | `identifier`                                                                                                     | [`schema:identifier`](https://schema.org/identifier)               | ([`PropertyValue`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property-value.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md))* | Any kind of identifier for any kind of Thing.                                                                  | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `images`             | `image`                                                                                                          | [`schema:image`](https://schema.org/image)                         | [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)*                                                                                                          | Images of the item.                                                                                            | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `name`               | -                                                                                                                | [`schema:name`](https://schema.org/name)                           | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The name of the item.                                                                                          | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `url`                | -                                                                                                                | [`schema:url`](https://schema.org/url)                             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The URL of the item.                                                                                           | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `emails`             | `email`                                                                                                          | [`schema:email`](https://schema.org/email)                         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                      | Email address for correspondence.                                                                              | -                                                                                                |
| `telephoneNumbers`   | `telephone`, `telephone-numbers`, `telephone_numbers`, `telephoneNumber`, `telephone-number`, `telephone_number` | [`schema:telephone`](https://schema.org/telephone)                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                      | Telephone numbers for the contact point.                                                                       | -                                                                                                |
| `availableLanguages` | `available-languages`, `available_languages`, `availableLanguage`, `available-language`, `available_language`    | [`schema:availableLanguage`](https://schema.org/availableLanguage) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                      | Languages (human not programming) in which it is possible to communicate with the organization/department etc. | -                                                                                                |

## Related

The `ContactPoint` type is related to these types:

- Parents: [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)
- Children: [`PostalAddress`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/postal-address.md)

## Formats

The `ContactPoint` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `ContactPoint` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ContactPoint.jsonld)
- [JSON Schema](https://stencila.org/ContactPoint.schema.json)
- Python class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/contact_point.py)
- Rust struct [`ContactPoint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/contact_point.rs)
- TypeScript class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/ts/src/types/ContactPoint.ts)

## Source

This documentation was generated from [`ContactPoint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ContactPoint.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).