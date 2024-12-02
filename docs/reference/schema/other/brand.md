# Brand

**A brand used by an organization or person for labeling a product, product group, or similar.**

**`@id`**: [`schema:Brand`](https://schema.org/Brand)

## Properties

The `Brand` type has these properties:

| Name             | Aliases                                                                                   | `@id`                                                      | Type                                                                                                                                                                                                                  | Description                                   | Inherited from                                                                                   |
| ---------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`             | -                                                                                         | [`schema:id`](https://schema.org/id)                       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The identifier for this item.                 | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `alternateNames` | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                      | Alternate names (aliases) for the item.       | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `description`    | -                                                                                         | [`schema:description`](https://schema.org/description)     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | A description of the item.                    | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `identifiers`    | `identifier`                                                                              | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property-value.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md))* | Any kind of identifier for any kind of Thing. | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `images`         | `image`                                                                                   | [`schema:image`](https://schema.org/image)                 | [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)*                                                                                                          | Images of the item.                           | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `name`           | -                                                                                         | [`schema:name`](https://schema.org/name)                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The name of the item.                         | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `url`            | -                                                                                         | [`schema:url`](https://schema.org/url)                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The URL of the item.                          | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `logo`           | -                                                                                         | [`schema:logo`](https://schema.org/logo)                   | [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)                                                                                                           | A logo associated with the brand.             | -                                                                                                |
| `reviews`        | `review`                                                                                  | [`schema:review`](https://schema.org/review)               | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                      | Reviews of the brand.                         | -                                                                                                |

## Related

The `Brand` type is related to these types:

- Parents: [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)
- Children: none

## Formats

The `Brand` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | ⚠️ High loss |            | 🔶 Beta              |       |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |       |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |       |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |       |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |       |

## Bindings

The `Brand` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Brand.jsonld)
- [JSON Schema](https://stencila.org/Brand.schema.json)
- Python class [`Brand`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/brand.py)
- Rust struct [`Brand`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/brand.rs)
- TypeScript class [`Brand`](https://github.com/stencila/stencila/blob/main/ts/src/types/Brand.ts)

## Source

This documentation was generated from [`Brand.yaml`](https://github.com/stencila/stencila/blob/main/schema/Brand.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
