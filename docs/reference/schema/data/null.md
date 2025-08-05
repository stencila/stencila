---
title: 'Null'
description: The null value.
config:
  publish:
    ghost:
      type: post
      slug: 'null'
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

# Formats

The `Null` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🔷 Low loss   |              |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🔷 Low loss   | 🔷 Low loss   |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🔷 Low loss   | 🔷 Low loss   |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🔷 Low loss   | 🔷 Low loss   |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🔷 Low loss   | 🔷 Low loss   |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | 🔷 Low loss   |              |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |         |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |         |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |         |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |         |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |         |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |         |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |         |
| [Apache Parquet](https://stencila.ghost.io/docs/reference/formats/parquet)          |              |              |         |
| [Apache Arrow](https://stencila.ghost.io/docs/reference/formats/arrow)              |              |              |         |
| [Microsoft Excel XLSX](https://stencila.ghost.io/docs/reference/formats/xlsx)       |              |              |         |
| [Microsoft Excel XLS](https://stencila.ghost.io/docs/reference/formats/xls)         |              |              |         |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |         |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |         |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |         |

# Bindings

The `Null` type is represented in:

- [JSON-LD](https://stencila.org/Null.jsonld)
- [JSON Schema](https://stencila.org/Null.schema.json)
- Python type [`Null`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/null.py)
- Rust type [`Null`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/null.rs)
- TypeScript type [`Null`](https://github.com/stencila/stencila/blob/main/ts/src/types/Null.ts)

# Source

This documentation was generated from [`Null.yaml`](https://github.com/stencila/stencila/blob/main/schema/Null.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
