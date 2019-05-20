# Product

The `Product` type allows you to provide details about a product such as the product brand, logo and ID. This type can be used as any kind of product that is not [`CreativeWork`](/CreativeWork) item.

## Example

The example below are based on a model of [astrolabe](https://en.wikipedia.org/wiki/Astrolabe).

```json
{
    "type": "Product",
    "brand": "Astro",
    "name": "Astrolabe",
    "logo": "http//www.product-astrolabe.com/logo.png",
    "productID" : "AA55"
}
```

## Related

### JATS

`Product` is analagous, and structurally similar to, the JATS [`<product>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/product.html) element which contains the metadata concerning one product (for example, a book, software package, website, or hardware component) discussed in an article.

## Google Structured Data

`Product` is compliant with the guidelines for structural data provided by [Google](https://developers.google.com/search/docs/data-types/product#product). In both cases, Stencila JASON and Google Structural Data, the properties of the elements are derived from [Schema.org Product](https://schema.org/Product).