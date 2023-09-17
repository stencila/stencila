---
title:
- type: Text
  value: Product
---

# Product

**Any offered product or service. For example, a pair of shoes;
a haircut; or an episode of a TV show streamed online.
**

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

| Name           | `@id`                                                      | Type                                                                                                                                                       | Description                                   | Inherited from                                                        |
| -------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | --------------------------------------------------------------------- |
| id             | [`schema:id`](https://schema.org/id)                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                  | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)   |
| alternateNames | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.       | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)     |
| description    | [`schema:description`](https://schema.org/description)     | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                    | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)     |
| identifiers    | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing. | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)     |
| images         | [`schema:image`](https://schema.org/image)                 | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                           | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)     |
| name           | [`schema:name`](https://schema.org/name)                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                         | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)     |
| url            | [`schema:url`](https://schema.org/url)                     | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                          | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)     |
| brands         | [`schema:brand`](https://schema.org/brand)                 | [`Brand`](https://stencila.dev/docs/reference/schema/other/brand)*                                                                                         | Brands that the product is labelled with.     | [`Product`](https://stencila.dev/docs/reference/schema/other/product) |
| logo           | [`schema:logo`](https://schema.org/logo)                   | [`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string)       | The logo of the product.                      | [`Product`](https://stencila.dev/docs/reference/schema/other/product) |
| productID      | [`schema:productID`](https://schema.org/productID)         | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | Product identification code.                  | [`Product`](https://stencila.dev/docs/reference/schema/other/product) |

## Related

The `Product` type is related to these types:

- Parents: [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)
- Children: none

## Formats

The `Product` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                                   |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ------------------------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                                                                                         |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<product>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/product) |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |                                                                                                         |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                                         |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                         |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                         |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                         |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                                         |

## Bindings

The `Product` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Product.jsonld)
- [JSON Schema](https://stencila.dev/Product.schema.json)
- Python class [`Product`](https://github.com/stencila/stencila/blob/main/python/stencila/types/product.py)
- Rust struct [`Product`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/product.rs)
- TypeScript class [`Product`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Product.ts)

## Source

This documentation was generated from [`Product.yaml`](https://github.com/stencila/stencila/blob/main/schema/Product.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).