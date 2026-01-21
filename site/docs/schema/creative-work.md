---
title: Creative Work
description: A creative work, including books, movies, photographs, software programs, etc.
---

This is an implementation, and extension, of schema.org [`CreativeWork`](https://schema.org/CreativeWork).
It extends schema.org `CreativeWork` by, adding several properties including `dateAccepted`
and `fundedBy`.


# Properties

The `CreativeWork` type has these properties:

| Name             | Description                                                                                                             | Type                                                                              | Inherited from          |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                                                                                           | [`String`](./string.md)                                                           | [`Entity`](./entity.md) |
| `alternateNames` | Alternate names (aliases) for the item.                                                                                 | [`String`](./string.md)*                                                          | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                                                                                              | [`String`](./string.md)                                                           | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing.                                                                           | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*              | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                                                                                                     | [`ImageObject`](./image-object.md)*                                               | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                                                                                                   | [`String`](./string.md)                                                           | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                                                                                                    | [`String`](./string.md)                                                           | [`Thing`](./thing.md)   |
| `workType`       | The type of `CreativeWork` (e.g. article, book, software application).                                                  | [`CreativeWorkType`](./creative-work-type.md)                                     | -                       |
| `doi`            | The work's Digital Object Identifier (https://doi.org/).                                                                | [`String`](./string.md)                                                           | -                       |
| `about`          | The subject matter of the content.                                                                                      | [`ThingVariant`](./thing-variant.md)*                                             | -                       |
| `abstract`       | A short description that summarizes a `CreativeWork`.                                                                   | [`Block`](./block.md)*                                                            | -                       |
| `authors`        | The authors of the `CreativeWork`.                                                                                      | [`Author`](./author.md)*                                                          | -                       |
| `provenance`     | A summary of the provenance of the content within the work.                                                             | [`ProvenanceCount`](./provenance-count.md)*                                       | -                       |
| `contributors`   | A secondary contributor to the `CreativeWork`.                                                                          | [`Author`](./author.md)*                                                          | -                       |
| `editors`        | People who edited the `CreativeWork`.                                                                                   | [`Person`](./person.md)*                                                          | -                       |
| `maintainers`    | The maintainers of the `CreativeWork`.                                                                                  | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | -                       |
| `comments`       | Comments about this creative work.                                                                                      | [`Comment`](./comment.md)*                                                        | -                       |
| `dateCreated`    | Date/time of creation.                                                                                                  | [`Date`](./date.md)                                                               | -                       |
| `dateReceived`   | Date/time that work was received.                                                                                       | [`Date`](./date.md)                                                               | -                       |
| `dateAccepted`   | Date/time of acceptance.                                                                                                | [`Date`](./date.md)                                                               | -                       |
| `dateModified`   | Date/time of most recent modification.                                                                                  | [`Date`](./date.md)                                                               | -                       |
| `datePublished`  | Date of first publication.                                                                                              | [`Date`](./date.md)                                                               | -                       |
| `funders`        | People or organizations that funded the `CreativeWork`.                                                                 | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | -                       |
| `fundedBy`       | Grants that funded the `CreativeWork`; reverse of `fundedItems`.                                                        | ([`Grant`](./grant.md) \| [`MonetaryGrant`](./monetary-grant.md))*                | -                       |
| `genre`          | Genre of the creative work, broadcast channel or group.                                                                 | [`String`](./string.md)*                                                          | -                       |
| `keywords`       | Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.  | [`String`](./string.md)*                                                          | -                       |
| `isPartOf`       | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [`CreativeWorkVariant`](./creative-work-variant.md)                               | -                       |
| `licenses`       | License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.         | ([`CreativeWorkVariant`](./creative-work-variant.md) \| [`String`](./string.md))* | -                       |
| `parts`          | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [`CreativeWorkVariant`](./creative-work-variant.md)*                              | -                       |
| `publisher`      | A publisher of the CreativeWork.                                                                                        | [`Person`](./person.md) \| [`Organization`](./organization.md)                    | -                       |
| `bibliography`   | A bibliography of references which may be cited in the work.                                                            | [`Bibliography`](./bibliography.md)                                               | -                       |
| `references`     | References to other creative works, such as another publication, web page, scholarly article, etc.                      | [`Reference`](./reference.md)*                                                    | -                       |
| `text`           | The textual content of this creative work.                                                                              | [`Text`](./text.md)                                                               | -                       |
| `title`          | The title of the creative work.                                                                                         | [`Inline`](./inline.md)*                                                          | -                       |
| `repository`     | URL of the repository where the un-compiled, human readable source of the work is located.                              | [`String`](./string.md)                                                           | -                       |
| `path`           | The file system path of the source of the work.                                                                         | [`String`](./string.md)                                                           | -                       |
| `commit`         | The commit hash (or similar) of the source of the work.                                                                 | [`String`](./string.md)                                                           | -                       |
| `version`        | The version of the creative work.                                                                                       | [`String`](./string.md) \| [`Number`](./number.md)                                | -                       |

# Related

The `CreativeWork` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: [`Article`](./article.md), [`Chat`](./chat.md), [`Claim`](./claim.md), [`Collection`](./collection.md), [`Comment`](./comment.md), [`Datatable`](./datatable.md), [`Figure`](./figure.md), [`File`](./file.md), [`MediaObject`](./media-object.md), [`Periodical`](./periodical.md), [`Prompt`](./prompt.md), [`PublicationIssue`](./publication-issue.md), [`PublicationVolume`](./publication-volume.md), [`Review`](./review.md), [`SoftwareApplication`](./software-application.md), [`SoftwareSourceCode`](./software-source-code.md), [`Table`](./table.md)

# Bindings

The `CreativeWork` type is represented in:

- [JSON-LD](https://stencila.org/CreativeWork.jsonld)
- [JSON Schema](https://stencila.org/CreativeWork.schema.json)
- Python class [`CreativeWork`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/creative_work.py)
- Rust struct [`CreativeWork`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/creative_work.rs)
- TypeScript class [`CreativeWork`](https://github.com/stencila/stencila/blob/main/ts/src/types/CreativeWork.ts)

# Source

This documentation was generated from [`CreativeWork.yaml`](https://github.com/stencila/stencila/blob/main/schema/CreativeWork.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
