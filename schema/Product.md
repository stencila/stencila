---
title: Product
authors: []
---

include: ../public/Product.schema.md
:::
Any offered product or service. For example, a pair of shoes; a concert ticket; the rental of a car; a haircut; or an episode of a TV show streamed online. https&#x3A;//schema.org/Product

| Entity  | type           | The name of the type and all descendant types.                                 | string |
| ------- | -------------- | ------------------------------------------------------------------------------ | ------ |
| Entity  | id             | The identifier for this item.                                                  | string |
| Thing   | alternateNames | Alternate names (aliases) for the item.                                        | array  |
| Thing   | description    | A description of the item.                                                     | string |
| Thing   | meta           | Metadata associated with this item.                                            | object |
| Thing   | name           | The name of the item.                                                          | string |
| Thing   | url            | The URL of the item.                                                           | string |
| Product | brand          | Brand that the product is labelled with.                                       |        |
|         |                |                                                                                |        |
| Product | logo           | A logo of of the product. It can be either a URL of the image or image itself. |        |
|         |                |                                                                                |        |
| Product | productID      | Product identification code.                                                   |        |
| string  |                |                                                                                |        |

:::

The `Product` type allows you to provide details about a product such as the product brand, logo and ID. This type can be used as any kind of product that is not [`CreativeWork`](/CreativeWork) item.

# Examples

The examples below are based on a model of [astrolabe](https://en.wikipedia.org/wiki/Astrolabe).

```json import=example
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

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## JATS

`Product` is analogous, and structurally similar to, the JATS [`<product>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/product.html) element which contains the metadata concerning one product (for example, a book, software package, website, or hardware component) discussed in an article.

```jats export=example


```

## Google Structured Data

To meet Google's guidelines for [`Product`](https://developers.google.com/search/docs/data-types/product#product) instances are required to have `image` and `name` properties.

[//]: # 'WIP: Needs JATS Fixes'
