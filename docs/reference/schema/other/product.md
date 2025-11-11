---
title: Product
description: Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
config:
  publish:
    ghost:
      type: post
      slug: product
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
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

| Name             | Description                                   | Type                                                                                                                                                       | Inherited from                                                     | `JSON-LD @id`                                              | Aliases                                                                                   |
| ---------------- | --------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`             | The identifier for this item.                 | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                       | -                                                                                         |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:alternateName`](https://schema.org/alternateName) | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`    | A description of the item.                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:description`](https://schema.org/description)     | -                                                                                         |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](https://stencila.ghost.io/docs/reference/schema/property-value) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string))* | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:identifier`](https://schema.org/identifier)       | `identifier`                                                                              |
| `images`         | Images of the item.                           | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*                                                                             | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:image`](https://schema.org/image)                 | `image`                                                                                   |
| `name`           | The name of the item.                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:name`](https://schema.org/name)                   | -                                                                                         |
| `url`            | The URL of the item.                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:url`](https://schema.org/url)                     | -                                                                                         |
| `brands`         | Brands that the product is labelled with.     | [`Brand`](https://stencila.ghost.io/docs/reference/schema/brand)*                                                                                          | -                                                                  | [`schema:brand`](https://schema.org/brand)                 | `brand`                                                                                   |
| `logo`           | The logo of the product.                      | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)                                                                              | -                                                                  | [`schema:logo`](https://schema.org/logo)                   | -                                                                                         |
| `productID`      | Product identification code.                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | [`schema:productID`](https://schema.org/productID)         | `product-id`, `product_id`                                                                |

# Related

The `Product` type is related to these types:

- Parents: [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)
- Children: none

# Formats

The `Product` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                                  | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | -------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                          |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                                                                                          |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🔷 Low loss   |              | Encoded as [`<product>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/product.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              |                                                                                                          |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |                                                                                                          |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |                                                                                                          |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |                                                                                                          |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |                                                                                                          |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                          |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                          |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [Microsoft Word](https://stencila.ghost.io/docs/reference/formats/docx)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/czst)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                          |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                          |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                                          |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                                          |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |                                                                                                          |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |                                                                                                          |
| [Microsoft Excel](https://stencila.ghost.io/docs/reference/formats/xlsx)            |              |              |                                                                                                          |
| [Microsoft Excel (XLS)](https://stencila.ghost.io/docs/reference/formats/xls)       |              |              |                                                                                                          |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |                                                                                                          |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                          |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                          |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                          |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                          |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              |              |                                                                                                          |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                          |

# Bindings

The `Product` type is represented in:

- [JSON-LD](https://stencila.org/Product.jsonld)
- [JSON Schema](https://stencila.org/Product.schema.json)
- Python class [`Product`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/product.py)
- Rust struct [`Product`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/product.rs)
- TypeScript class [`Product`](https://github.com/stencila/stencila/blob/main/ts/src/types/Product.ts)

# Source

This documentation was generated from [`Product.yaml`](https://github.com/stencila/stencila/blob/main/schema/Product.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
