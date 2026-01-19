---
title: Table
description: A table.
---

# Properties

The `Table` type has these properties:

| Name                 | Description                                                                                                             | Type                                                                              | Inherited from                       | `JSON-LD @id`                                                | Aliases                                                                                   |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| `id`                 | The identifier for this item.                                                                                           | [`String`](./string.md)                                                           | [`Entity`](./entity.md)              | [`schema:id`](https://schema.org/id)                         | -                                                                                         |
| `alternateNames`     | Alternate names (aliases) for the item.                                                                                 | [`String`](./string.md)*                                                          | [`Thing`](./thing.md)                | [`schema:alternateName`](https://schema.org/alternateName)   | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`        | A description of the item.                                                                                              | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                | [`schema:description`](https://schema.org/description)       | -                                                                                         |
| `identifiers`        | Any kind of identifier for any kind of Thing.                                                                           | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*              | [`Thing`](./thing.md)                | [`schema:identifier`](https://schema.org/identifier)         | `identifier`                                                                              |
| `images`             | Images of the item.                                                                                                     | [`ImageObject`](./image-object.md)*                                               | [`Thing`](./thing.md)                | [`schema:image`](https://schema.org/image)                   | `image`                                                                                   |
| `name`               | The name of the item.                                                                                                   | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                | [`schema:name`](https://schema.org/name)                     | -                                                                                         |
| `url`                | The URL of the item.                                                                                                    | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                | [`schema:url`](https://schema.org/url)                       | -                                                                                         |
| `workType`           | The type of `CreativeWork` (e.g. article, book, software application).                                                  | [`CreativeWorkType`](./creative-work-type.md)                                     | [`CreativeWork`](./creative-work.md) | `stencila:workType`                                          | `work-type`, `work_type`                                                                  |
| `doi`                | The work's Digital Object Identifier (https://doi.org/).                                                                | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) | `stencila:doi`                                               | -                                                                                         |
| `about`              | The subject matter of the content.                                                                                      | [`ThingVariant`](./thing-variant.md)*                                             | [`CreativeWork`](./creative-work.md) | [`schema:about`](https://schema.org/about)                   | -                                                                                         |
| `abstract`           | A short description that summarizes a `CreativeWork`.                                                                   | [`Block`](./block.md)*                                                            | [`CreativeWork`](./creative-work.md) | [`schema:abstract`](https://schema.org/abstract)             | -                                                                                         |
| `authors`            | The authors of the `CreativeWork`.                                                                                      | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) | [`schema:author`](https://schema.org/author)                 | `author`                                                                                  |
| `provenance`         | A summary of the provenance of the content within the work.                                                             | [`ProvenanceCount`](./provenance-count.md)*                                       | [`CreativeWork`](./creative-work.md) | `stencila:provenance`                                        | -                                                                                         |
| `contributors`       | A secondary contributor to the `CreativeWork`.                                                                          | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) | [`schema:contributor`](https://schema.org/contributor)       | `contributor`                                                                             |
| `editors`            | People who edited the `CreativeWork`.                                                                                   | [`Person`](./person.md)*                                                          | [`CreativeWork`](./creative-work.md) | [`schema:editor`](https://schema.org/editor)                 | `editor`                                                                                  |
| `maintainers`        | The maintainers of the `CreativeWork`.                                                                                  | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) | [`schema:maintainer`](https://schema.org/maintainer)         | `maintainer`                                                                              |
| `comments`           | Comments about this creative work.                                                                                      | [`Comment`](./comment.md)*                                                        | [`CreativeWork`](./creative-work.md) | [`schema:comment`](https://schema.org/comment)               | `comment`                                                                                 |
| `dateCreated`        | Date/time of creation.                                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) | [`schema:dateCreated`](https://schema.org/dateCreated)       | `date-created`, `date_created`                                                            |
| `dateReceived`       | Date/time that work was received.                                                                                       | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) | [`schema:dateReceived`](https://schema.org/dateReceived)     | `date-received`, `date_received`                                                          |
| `dateAccepted`       | Date/time of acceptance.                                                                                                | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) | `stencila:dateAccepted`                                      | `date-accepted`, `date_accepted`                                                          |
| `dateModified`       | Date/time of most recent modification.                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) | [`schema:dateModified`](https://schema.org/dateModified)     | `date-modified`, `date_modified`                                                          |
| `datePublished`      | Date of first publication.                                                                                              | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) | [`schema:datePublished`](https://schema.org/datePublished)   | `date`, `date-published`, `date_published`                                                |
| `funders`            | People or organizations that funded the `CreativeWork`.                                                                 | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) | [`schema:funder`](https://schema.org/funder)                 | `funder`                                                                                  |
| `fundedBy`           | Grants that funded the `CreativeWork`; reverse of `fundedItems`.                                                        | ([`Grant`](./grant.md) \| [`MonetaryGrant`](./monetary-grant.md))*                | [`CreativeWork`](./creative-work.md) | `stencila:fundedBy`                                          | `funded-by`, `funded_by`                                                                  |
| `genre`              | Genre of the creative work, broadcast channel or group.                                                                 | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) | [`schema:genre`](https://schema.org/genre)                   | -                                                                                         |
| `keywords`           | Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.  | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) | [`schema:keywords`](https://schema.org/keywords)             | `keyword`                                                                                 |
| `isPartOf`           | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [`CreativeWorkVariant`](./creative-work-variant.md)                               | [`CreativeWork`](./creative-work.md) | [`schema:isPartOf`](https://schema.org/isPartOf)             | `is-part-of`, `is_part_of`                                                                |
| `licenses`           | License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.         | ([`CreativeWorkVariant`](./creative-work-variant.md) \| [`String`](./string.md))* | [`CreativeWork`](./creative-work.md) | [`schema:license`](https://schema.org/license)               | `license`                                                                                 |
| `parts`              | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [`CreativeWorkVariant`](./creative-work-variant.md)*                              | [`CreativeWork`](./creative-work.md) | [`schema:hasParts`](https://schema.org/hasParts)             | `hasParts`, `part`                                                                        |
| `publisher`          | A publisher of the CreativeWork.                                                                                        | [`Person`](./person.md) \| [`Organization`](./organization.md)                    | [`CreativeWork`](./creative-work.md) | [`schema:publisher`](https://schema.org/publisher)           | -                                                                                         |
| `bibliography`       | A bibliography of references which may be cited in the work.                                                            | [`Bibliography`](./bibliography.md)                                               | [`CreativeWork`](./creative-work.md) | `stencila:bibliography`                                      | -                                                                                         |
| `references`         | References to other creative works, such as another publication, web page, scholarly article, etc.                      | [`Reference`](./reference.md)*                                                    | [`CreativeWork`](./creative-work.md) | [`schema:citation`](https://schema.org/citation)             | `citations`, `reference`                                                                  |
| `text`               | The textual content of this creative work.                                                                              | [`Text`](./text.md)                                                               | [`CreativeWork`](./creative-work.md) | [`schema:text`](https://schema.org/text)                     | -                                                                                         |
| `title`              | The title of the creative work.                                                                                         | [`Inline`](./inline.md)*                                                          | [`CreativeWork`](./creative-work.md) | [`schema:headline`](https://schema.org/headline)             | `headline`                                                                                |
| `repository`         | URL of the repository where the un-compiled, human readable source of the work is located.                              | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) | [`schema:codeRepository`](https://schema.org/codeRepository) | -                                                                                         |
| `path`               | The file system path of the source of the work.                                                                         | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) | `stencila:path`                                              | -                                                                                         |
| `commit`             | The commit hash (or similar) of the source of the work.                                                                 | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) | `stencila:commit`                                            | -                                                                                         |
| `version`            | The version of the creative work.                                                                                       | [`String`](./string.md) \| [`Number`](./number.md)                                | [`CreativeWork`](./creative-work.md) | [`schema:version`](https://schema.org/version)               | -                                                                                         |
| `label`              | A short label for the table.                                                                                            | [`String`](./string.md)                                                           | -                                    | `stencila:label`                                             | -                                                                                         |
| `labelAutomatically` | Whether the label should be automatically updated.                                                                      | [`Boolean`](./boolean.md)                                                         | -                                    | `stencila:labelAutomatically`                                | `label-automatically`, `label_automatically`                                              |
| `caption`            | A caption for the table.                                                                                                | [`Block`](./block.md)*                                                            | -                                    | [`schema:caption`](https://schema.org/caption)               | -                                                                                         |
| `rows`               | Rows of cells in the table.                                                                                             | [`TableRow`](./table-row.md)*                                                     | -                                    | `stencila:rows`                                              | `row`                                                                                     |
| `notes`              | Notes for the table.                                                                                                    | [`Block`](./block.md)*                                                            | -                                    | [`schema:notes`](https://schema.org/notes)                   | `note`                                                                                    |

# Related

The `Table` type is related to these types:

- Parents: [`CreativeWork`](./creative-work.md)
- Children: none

# Formats

The `Table` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded using special function     |
| [JATS](../formats/jats.md)                       |              |              | Encoded using special function     |
| [Markdown](../formats/md.md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function |
| [Stencila Markdown](../formats/smd.md)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Quarto Markdown](../formats/qmd.md)             | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [MyST Markdown](../formats/myst.md)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [LLM Markdown](../formats/llmd.md)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                    |
| [CSV](../formats/csv.md)                         |              |              |                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                    |
| [Directory](../formats/directory.md)             |              |              |                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                    |

# Bindings

The `Table` type is represented in:

- [JSON-LD](https://stencila.org/Table.jsonld)
- [JSON Schema](https://stencila.org/Table.schema.json)
- Python class [`Table`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/table.py)
- Rust struct [`Table`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table.rs)
- TypeScript class [`Table`](https://github.com/stencila/stencila/blob/main/ts/src/types/Table.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Table` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                 | Strategy                                        |
| --------- | ---------- | ----------------------------------------------------------- | ----------------------------------------------- |
| `caption` | Min+       | No caption.                                                 | `None`                                          |
|           | Low+       | Generate up to two arbitrary paragraphs.                    | `option::of(vec_paragraphs(2))`                 |
|           | Max        | Generate up to three arbitrary, non-recursive, block nodes. | `option::of(vec_blocks_non_recursive(3))`       |
| `rows`    | Min+       | Generate up to a 2x2 table with a header row.               | `table_rows_with_header(2,2)`                   |
|           | Low+       | Generate up to a 3x3 table with a header row.               | `table_rows_with_header(3,3)`                   |
|           | High+      | Generate up to four, arbitrary, table rows.                 | `vec(TableRow::arbitrary(), size_range(1..=4))` |
|           | Max        | Generate up to eight, arbitrary, table rows.                | `vec(TableRow::arbitrary(), size_range(1..=8))` |
| `notes`   | Min+       | No notes.                                                   | `None`                                          |
|           | Low+       | Generate an arbitrary paragraph.                            | `option::of(vec_paragraphs(1))`                 |
|           | Max        | Generate up to two arbitrary, non-recursive, block nodes.   | `option::of(vec_blocks_non_recursive(2))`       |

# Source

This documentation was generated from [`Table.yaml`](https://github.com/stencila/stencila/blob/main/schema/Table.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
