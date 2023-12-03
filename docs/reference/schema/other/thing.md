# Thing

**The most generic type of item.**

**`@id`**: [`schema:Thing`](https://schema.org/Thing)

## Properties

The `Thing` type has these properties:

| Name             | Aliases                                                                                   | `@id`                                                      | Type                                                                                                                                                                                                                 | Description                                   | Inherited from                                                                                   |
| ---------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`             | -                                                                                         | [`schema:id`](https://schema.org/id)                       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                      | The identifier for this item.                 | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `alternateNames` | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                     | Alternate names (aliases) for the item.       | -                                                                                                |
| `description`    | -                                                                                         | [`schema:description`](https://schema.org/description)     | [`Text`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md)                                                                                                                         | A description of the item.                    | -                                                                                                |
| `identifiers`    | `identifier`                                                                              | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property-value.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md))* | Any kind of identifier for any kind of Thing. | -                                                                                                |
| `images`         | `image`                                                                                   | [`schema:image`](https://schema.org/image)                 | [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)*                                                                                                         | Images of the item.                           | -                                                                                                |
| `name`           | -                                                                                         | [`schema:name`](https://schema.org/name)                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                      | The name of the item.                         | -                                                                                                |
| `url`            | -                                                                                         | [`schema:url`](https://schema.org/url)                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                      | The URL of the item.                          | -                                                                                                |

## Related

The `Thing` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`Brand`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/brand.md), [`ContactPoint`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/contact-point.md), [`CreativeWork`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/creative-work.md), [`DatatableColumn`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable-column.md), [`DefinedTerm`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/defined-term.md), [`Enumeration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/enumeration.md), [`Grant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/grant.md), [`ListItem`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list-item.md), [`Organization`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md), [`Person`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md), [`Product`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/product.md), [`PropertyValue`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property-value.md)

## Formats

The `Thing` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |                  |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss    | 🔶 Beta                 |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |       |

## Bindings

The `Thing` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Thing.jsonld)
- [JSON Schema](https://stencila.org/Thing.schema.json)
- Python class [`Thing`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/thing.py)
- Rust struct [`Thing`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/thing.rs)
- TypeScript class [`Thing`](https://github.com/stencila/stencila/blob/main/ts/src/types/Thing.ts)

## Source

This documentation was generated from [`Thing.yaml`](https://github.com/stencila/stencila/blob/main/schema/Thing.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).