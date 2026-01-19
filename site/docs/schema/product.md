---
title: Product
description: Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
---

The `Product` type allows you to provide details about a product such as the product
brand, logo and ID. This type can be used as any kind of product that is not [`CreativeWork`](./CreativeWork) item.

`Product` is analogous, and structurally similar to, the 
JATS XML [`<product>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/product.html) element which
contains the metadata concerning one product (for example, a book, software package, website, or
hardware component) discussed in an article.

To meet Google's guidelines for [`Product`](https://developers.google.com/search/docs/data-types/product#product)
instances are required to have `image` and `name` properties.


# Properties

The `Product` type has these properties:

| Name             | Description                                   | Type                                                                 | Inherited from          | `JSON-LD @id`                                              | Aliases                                                                                   |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------- | ----------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`             | The identifier for this item.                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                       | -                                                                                         |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   | [`schema:alternateName`](https://schema.org/alternateName) | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`    | A description of the item.                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:description`](https://schema.org/description)     | -                                                                                         |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   | [`schema:identifier`](https://schema.org/identifier)       | `identifier`                                                                              |
| `images`         | Images of the item.                           | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   | [`schema:image`](https://schema.org/image)                 | `image`                                                                                   |
| `name`           | The name of the item.                         | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:name`](https://schema.org/name)                   | -                                                                                         |
| `url`            | The URL of the item.                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:url`](https://schema.org/url)                     | -                                                                                         |
| `brands`         | Brands that the product is labelled with.     | [`Brand`](./brand.md)*                                               | -                       | [`schema:brand`](https://schema.org/brand)                 | `brand`                                                                                   |
| `logo`           | The logo of the product.                      | [`ImageObject`](./image-object.md)                                   | -                       | [`schema:logo`](https://schema.org/logo)                   | -                                                                                         |
| `productID`      | Product identification code.                  | [`String`](./string.md)                                              | -                       | [`schema:productID`](https://schema.org/productID)         | `product-id`, `product_id`                                                                |

# Related

The `Product` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Formats

The `Product` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                  | Notes |
| ------------------------------------------------ | ------------ | ------------ | -------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                          |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                                                                                          |
| [JATS](../formats/jats.md)                       | 🔷 Low loss   |              | Encoded as [`<product>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/product.html) |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              |                                                                                                          |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                                                                                          |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                                                                                          |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                                                                                          |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                                                                                          |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                          |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                          |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                          |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                          |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                          |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                          |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                          |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                          |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                          |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                          |
| [Directory](../formats/directory.md)             |              |              |                                                                                                          |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                          |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                          |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                          |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                          |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                          |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                          |

# Bindings

The `Product` type is represented in:

- [JSON-LD](https://stencila.org/Product.jsonld)
- [JSON Schema](https://stencila.org/Product.schema.json)
- Python class [`Product`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/product.py)
- Rust struct [`Product`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/product.rs)
- TypeScript class [`Product`](https://github.com/stencila/stencila/blob/main/ts/src/types/Product.ts)

# Source

This documentation was generated from [`Product.yaml`](https://github.com/stencila/stencila/blob/main/schema/Product.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
