# Product

**Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.**

The `Product` type allows you to provide details about a product such as the product brand, logo and ID. This type can be used as any kind of product that is not [`CreativeWork`](./CreativeWork) item. `Product` is analogous, and structurally similar to, the JATS XML [`<product>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/product.html) element which contains the metadata concerning one product (for example, a book, software package, website, or hardware component) discussed in an article. To meet Google's guidelines for [`Product`](https://developers.google.com/search/docs/data-types/product#product) instances are required to have `image` and `name` properties.

## Properties

| Name           | `@id`                                                    | Type                                                                                                 | Description                                                         | Inherited from        |
| -------------- | -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | --------------------- |
| alternateNames | [schema:alternateName](https://schema.org/alternateName) | Array of string                                                                                      | Alternate names (aliases) for the item.                             | [Thing](Thing.md)     |
| brands         | [schema:brand](https://schema.org/brand)                 | Array of [Brand](Brand.md)                                                                           | Brands that the product is labelled with.                           | [Product](Product.md) |
| description    | [schema:description](https://schema.org/description)     | Array of [BlockContent](BlockContent.md) _or_ Array of [InlineContent](InlineContent.md) _or_ string | A description of the item. See note [1](#notes).                    | [Thing](Thing.md)     |
| id             | [schema:id](https://schema.org/id)                       | string                                                                                               | The identifier for this item.                                       | [Entity](Entity.md)   |
| identifiers    | [schema:identifier](https://schema.org/identifier)       | Array of ([PropertyValue](PropertyValue.md) _or_ string)                                             | Any kind of identifier for any kind of Thing. See note [2](#notes). | [Thing](Thing.md)     |
| images         | [schema:image](https://schema.org/image)                 | Array of ([ImageObject](ImageObject.md) _or_ Format 'uri')                                           | Images of the item.                                                 | [Thing](Thing.md)     |
| logo           | [schema:logo](https://schema.org/logo)                   | [ImageObject](ImageObject.md) _or_ Format 'uri'                                                      | The logo of the product.                                            | [Product](Product.md) |
| meta           | [stencila:meta](https://schema.stenci.la/meta.jsonld)    | object                                                                                               | Metadata associated with this item.                                 | [Entity](Entity.md)   |
| name           | [schema:name](https://schema.org/name)                   | string                                                                                               | The name of the item.                                               | [Thing](Thing.md)     |
| productID      | [schema:productID](https://schema.org/productID)         | string                                                                                               | Product identification code.                                        | [Product](Product.md) |
| url            | [schema:url](https://schema.org/url)                     | Format 'uri'                                                                                         | The URL of the item.                                                | [Thing](Thing.md)     |

## Notes

1. **description** : Allows for the description to be an array of nodes (e.g. an array of inline content, or a couple of paragraphs), or a string. The `minItems` restriction avoids a string being coerced into an array with a single string item.
2. **identifiers** : Some identifiers have specific properties e.g the `issn` property for the `Periodical` type. These should be used in preference to this property which is intended for identifiers that do not yet have a specific property. Identifiers can be represented as strings, but using a `PropertyValue` will usually be better because it allows for `propertyID` (i.e. the type of identifier).

## Examples

```json
{
  "type": "Product",
  "brand": {
    "type": "Brand",
    "name": "Astro"
  },
  "name": "Astrolabe",
  "logo": {
    "type": "ImageObject",
    "contentUrl": "http://www.product-astrolabe.com/logo.png",
    "caption": "Astrolabe Logo"
  },
  "productID": "AA55"
}
```

## Related

- Parent: [Thing](Thing.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Product.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Product.schema.json)
- Python [`class Product`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Product)
- TypeScript [`interface Product`](https://stencila.github.io/schema/ts/docs/interfaces/product.html)
- R [`class Product`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Product`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Product.html)

## Source

This documentation was generated from [Product.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/Product.schema.yaml).
