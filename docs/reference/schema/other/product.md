# Product

**Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.**

The `Product` type allows you to provide details about a product such as the product
brand, logo and ID. This type can be used as any kind of product that is not [`CreativeWork`](./CreativeWork) item.

`Product` is analogous, and structurally similar to, the 
JATS XML [`<product>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/product.html) element which
contains the metadata concerning one product (for example, a book, software package, website, or
hardware component) discussed in an article.

To meet Google's guidelines for [`Product`](https://developers.google.com/search/docs/data-types/product#product)
instances are required to have `image` and `name` properties.


**`@id`**: [`schema:Product`](https://schema.org/Product)

## Properties

The `Product` type has these properties:

| Name             | Aliases                                                                                   | `@id`                                                      | Type                                                                                                                                                                                                                  | Description                                   | Inherited from                                                                                   |
| ---------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`             | -                                                                                         | [`schema:id`](https://schema.org/id)                       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The identifier for this item.                 | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `alternateNames` | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                                                                                                                      | Alternate names (aliases) for the item.       | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `description`    | -                                                                                         | [`schema:description`](https://schema.org/description)     | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                                                                                                                           | A description of the item.                    | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `identifiers`    | `identifier`                                                                              | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property-value.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md))* | Any kind of identifier for any kind of Thing. | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `images`         | `image`                                                                                   | [`schema:image`](https://schema.org/image)                 | [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)*                                                                                                          | Images of the item.                           | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `name`           | -                                                                                         | [`schema:name`](https://schema.org/name)                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The name of the item.                         | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `url`            | -                                                                                         | [`schema:url`](https://schema.org/url)                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | The URL of the item.                          | [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)   |
| `brands`         | `brand`                                                                                   | [`schema:brand`](https://schema.org/brand)                 | [`Brand`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/brand.md)*                                                                                                                       | Brands that the product is labelled with.     | -                                                                                                |
| `logo`           | -                                                                                         | [`schema:logo`](https://schema.org/logo)                   | [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)                                                                                                           | The logo of the product.                      | -                                                                                                |
| `productID`      | `product-id`, `product_id`                                                                | [`schema:productID`](https://schema.org/productID)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                       | Product identification code.                  | -                                                                                                |

## Related

The `Product` type is related to these types:

- Parents: [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)
- Children: none

## Formats

The `Product` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes                                                                                                    |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | -------------------------------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |                                                                                                          |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |                                                                                                          |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🔷 Low loss   |           | 🚧 Under development | Encoded as [`<product>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/product.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            |                                                                                                          |
| [MyST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)                | ⚠️ High loss |           | ⚠️ Alpha            |                                                                                                          |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |                                                                                                          |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                          |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                          |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                                                                                          |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                          |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                          |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                          |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |                                                                                                          |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |           | 🚧 Under development |                                                                                                          |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |                                                                                                          |

## Bindings

The `Product` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Product.jsonld)
- [JSON Schema](https://stencila.org/Product.schema.json)
- Python class [`Product`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/product.py)
- Rust struct [`Product`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/product.rs)
- TypeScript class [`Product`](https://github.com/stencila/stencila/blob/main/ts/src/types/Product.ts)

## Source

This documentation was generated from [`Product.yaml`](https://github.com/stencila/stencila/blob/main/schema/Product.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
