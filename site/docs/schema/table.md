---
title: Table
description: A table.
---

# Properties

The `Table` type has these properties:

| Name                 | Description                                                                                                             | Type                                                                              | Inherited from                       |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | ------------------------------------ |
| `id`                 | The identifier for this item.                                                                                           | [`String`](./string.md)                                                           | [`Entity`](./entity.md)              |
| `alternateNames`     | Alternate names (aliases) for the item.                                                                                 | [`String`](./string.md)*                                                          | [`Thing`](./thing.md)                |
| `description`        | A description of the item.                                                                                              | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `identifiers`        | Any kind of identifier for any kind of Thing.                                                                           | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*              | [`Thing`](./thing.md)                |
| `images`             | Images of the item.                                                                                                     | [`ImageObject`](./image-object.md)*                                               | [`Thing`](./thing.md)                |
| `name`               | The name of the item.                                                                                                   | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `url`                | The URL of the item.                                                                                                    | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `workType`           | The type of `CreativeWork` (e.g. article, book, software application).                                                  | [`CreativeWorkType`](./creative-work-type.md)                                     | [`CreativeWork`](./creative-work.md) |
| `doi`                | The work's Digital Object Identifier (https://doi.org/).                                                                | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `about`              | The subject matter of the content.                                                                                      | [`ThingVariant`](./thing-variant.md)*                                             | [`CreativeWork`](./creative-work.md) |
| `abstract`           | A short description that summarizes a `CreativeWork`.                                                                   | [`Block`](./block.md)*                                                            | [`CreativeWork`](./creative-work.md) |
| `authors`            | The authors of the `CreativeWork`.                                                                                      | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `provenance`         | A summary of the provenance of the content within the work.                                                             | [`ProvenanceCount`](./provenance-count.md)*                                       | [`CreativeWork`](./creative-work.md) |
| `contributors`       | A secondary contributor to the `CreativeWork`.                                                                          | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `editors`            | People who edited the `CreativeWork`.                                                                                   | [`Person`](./person.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `maintainers`        | The maintainers of the `CreativeWork`.                                                                                  | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `comments`           | Comments about this creative work.                                                                                      | [`Comment`](./comment.md)*                                                        | [`CreativeWork`](./creative-work.md) |
| `dateCreated`        | Date/time of creation.                                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateReceived`       | Date/time that work was received.                                                                                       | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateAccepted`       | Date/time of acceptance.                                                                                                | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateModified`       | Date/time of most recent modification.                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `datePublished`      | Date of first publication.                                                                                              | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `funders`            | People or organizations that funded the `CreativeWork`.                                                                 | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `fundedBy`           | Grants that funded the `CreativeWork`; reverse of `fundedItems`.                                                        | ([`Grant`](./grant.md) \| [`MonetaryGrant`](./monetary-grant.md))*                | [`CreativeWork`](./creative-work.md) |
| `genre`              | Genre of the creative work, broadcast channel or group.                                                                 | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `keywords`           | Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.  | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `isPartOf`           | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [`CreativeWorkVariant`](./creative-work-variant.md)                               | [`CreativeWork`](./creative-work.md) |
| `licenses`           | License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.         | ([`CreativeWorkVariant`](./creative-work-variant.md) \| [`String`](./string.md))* | [`CreativeWork`](./creative-work.md) |
| `parts`              | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [`CreativeWorkVariant`](./creative-work-variant.md)*                              | [`CreativeWork`](./creative-work.md) |
| `publisher`          | A publisher of the CreativeWork.                                                                                        | [`Person`](./person.md) \| [`Organization`](./organization.md)                    | [`CreativeWork`](./creative-work.md) |
| `bibliography`       | A bibliography of references which may be cited in the work.                                                            | [`Bibliography`](./bibliography.md)                                               | [`CreativeWork`](./creative-work.md) |
| `references`         | References to other creative works, such as another publication, web page, scholarly article, etc.                      | [`Reference`](./reference.md)*                                                    | [`CreativeWork`](./creative-work.md) |
| `text`               | The textual content of this creative work.                                                                              | [`Text`](./text.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `title`              | The title of the creative work.                                                                                         | [`Inline`](./inline.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `repository`         | URL of the repository where the un-compiled, human readable source of the work is located.                              | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `path`               | The file system path of the source of the work.                                                                         | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `commit`             | The commit hash (or similar) of the source of the work.                                                                 | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `version`            | The version of the creative work.                                                                                       | [`String`](./string.md) \| [`Number`](./number.md)                                | [`CreativeWork`](./creative-work.md) |
| `label`              | A short label for the table.                                                                                            | [`String`](./string.md)                                                           | -                                    |
| `labelAutomatically` | Whether the label should be automatically updated.                                                                      | [`Boolean`](./boolean.md)                                                         | -                                    |
| `caption`            | A caption for the table.                                                                                                | [`Block`](./block.md)*                                                            | -                                    |
| `rows`               | Rows of cells in the table.                                                                                             | [`TableRow`](./table-row.md)*                                                     | -                                    |
| `notes`              | Notes for the table.                                                                                                    | [`Block`](./block.md)*                                                            | -                                    |

# Related

The `Table` type is related to these types:

- Parents: [`CreativeWork`](./creative-work.md)
- Children: none

# Bindings

The `Table` type is represented in:

- [JSON-LD](https://stencila.org/Table.jsonld)
- [JSON Schema](https://stencila.org/Table.schema.json)
- Python class [`Table`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Table`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table.rs)
- TypeScript class [`Table`](https://github.com/stencila/stencila/blob/main/ts/src/types/Table.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Table` type are generated using the following strategies.

::: table

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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Table.yaml`](https://github.com/stencila/stencila/blob/main/schema/Table.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
