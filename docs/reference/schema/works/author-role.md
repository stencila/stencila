---
title: Author Role
description: An author and their role.
config:
  publish:
    ghost:
      type: post
      slug: author-role
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

# Properties

The `AuthorRole` type has these properties:

| Name           | Description                                                                | Type                                                                                     | Inherited from                                                     | `JSON-LD @id`                                    | Aliases                          |
| -------------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------ | -------------------------------- |
| `id`           | The identifier for this item.                                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                       | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)             | -                                |
| `author`       | The entity acting as an author.                                            | [`AuthorRoleAuthor`](https://stencila.ghost.io/docs/reference/schema/author-role-author) | -                                                                  | [`schema:author`](https://schema.org/author)     | -                                |
| `roleName`     | The role played by the author.                                             | [`AuthorRoleName`](https://stencila.ghost.io/docs/reference/schema/author-role-name)     | -                                                                  | [`schema:roleName`](https://schema.org/roleName) | `role-name`, `role_name`         |
| `format`       | The format that the author used to perform the role. e.g. Markdown, Python | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                       | -                                                                  | `stencila:format`                                | -                                |
| `lastModified` | Timestamp of most recent modification, by the author, in the role.         | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                 | -                                                                  | `stencila:lastModified`                          | `last-modified`, `last_modified` |

# Related

The `AuthorRole` type is related to these types:

- Parents: [`Role`](https://stencila.ghost.io/docs/reference/schema/role)
- Children: none

# Formats

The `AuthorRole` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [Microsoft Word](https://stencila.ghost.io/docs/reference/formats/docx)             | 🔷 Low loss   | 🔷 Low loss   |         |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/czst)                  | 🟢 No loss    | 🟢 No loss    |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |         |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |         |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |         |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |         |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |         |
| [Microsoft Excel](https://stencila.ghost.io/docs/reference/formats/xlsx)            |              |              |         |
| [Microsoft Excel (XLS)](https://stencila.ghost.io/docs/reference/formats/xls)       |              |              |         |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |         |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |         |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              |              |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |         |

# Bindings

The `AuthorRole` type is represented in:

- [JSON-LD](https://stencila.org/AuthorRole.jsonld)
- [JSON Schema](https://stencila.org/AuthorRole.schema.json)
- Python class [`AuthorRole`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/author_role.py)
- Rust struct [`AuthorRole`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author_role.rs)
- TypeScript class [`AuthorRole`](https://github.com/stencila/stencila/blob/main/ts/src/types/AuthorRole.ts)

# Source

This documentation was generated from [`AuthorRole.yaml`](https://github.com/stencila/stencila/blob/main/schema/AuthorRole.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
