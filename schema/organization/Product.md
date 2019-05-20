# Product

The `Product` type allows you to provide details about a product such as the product brand, logo and ID. This type can be used as any kind of product that is not [`CreativeWork`](/CreativeWork) item.

## Example

The example below are based on a model of [astrolabe](https://en.wikipedia.org/wiki/Astrolabe).

```json
{
  "type": "Product",
  "brand": {
    "type": "Brand",
    "name": "Astro"
  },
  "name": "Astrolabe",
  "logo": "http//www.product-astrolabe.com/logo.png",
  "productID": "AA55"
}
```

## Related

### JATS

`Product` is analagous, and structurally similar to, the JATS [`<product>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/product.html) element which contains the metadata concerning one product (for example, a book, software package, website, or hardware component) discussed in an article.

## Google Structured Data

To meet Google's guidelines for [`Product`](https://developers.google.com/search/docs/data-types/product#product) instances are required to have `image` and `name` properties.
