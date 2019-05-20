# Product

The `Product` type allows you to provide details about a product such as the product brand, logo and ID. This type can be used as any kind of product that is not [`CreativeWork`](/CreativeWork) item.

## Examples

The examples below are based on a model of [direct vision spectroscope](https://www.eiscolabs.com/collections/spectroscopes/products/ph0595).

```json
{
    "type": "Product",
    "brand": "eisco",
    "logo": "https://beta-static.fishersci.com/content/dam/fishersci/en_US/images/brands/e/eisco/eisco-logo-1071.png",
    "productID" : "PH0595"
}
```

YAML provides a more readable format for providing details about a product in places like Markdown front-matter.

```markdown
---
title: Laboratory Catalogue
content:
  - product:
      - brand: eisco
        logo: https://beta-static.fishersci.com/content/dam/fishersci/en_US/images/brands/e/eisco/eisco-logo-1071.png
        productID: PH0595
---
```


## Related

### JATS

`Product` is analagous, and structurally similar to, the JATS [`<product>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/product.html) element which contains the metadata concerning one product (for example, a book, software package, website, or hardware component) discussed in an article.
