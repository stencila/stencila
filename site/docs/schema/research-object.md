---
title: Research Object
description: An abstract base type for research objects represented as block content.
---

This is an abstract base type used in Stencila Schema for authored research
objects that can be identified, cited, related in research discourse graphs,
and represented directly as block-level document content.

It provides shared `label`, `content`, `relations`, and `extra` properties
for concrete research object nodes such as `Claim`, `Question`, `Request`,
`Protocol`, and `Evidence`.


# Properties

The `ResearchObject` type has these properties:

| Name             | Description                                                                                                             | Type                                                                              | Inherited from                       |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | ------------------------------------ |
| `label`          | A short label for the research object.                                                                                  | [`String`](./string.md)                                                           | -                                    |
| `content`        | Content of the research object.                                                                                         | [`Block`](./block.md)*                                                            | -                                    |
| `relations`      | Relations from this research object to other research objects.                                                          | [`ResearchObjectRelation`](./research-object-relation.md)*                        | -                                    |
| `extra`          | Additional metadata for the research object.                                                                            | [`Object`](./object.md)                                                           | -                                    |
| `workType`       | The type of `CreativeWork` (e.g. article, book, software application).                                                  | [`CreativeWorkType`](./creative-work-type.md)                                     | [`CreativeWork`](./creative-work.md) |
| `doi`            | The work's Digital Object Identifier (https://doi.org/).                                                                | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `about`          | The subject matter of the content.                                                                                      | [`ThingVariant`](./thing-variant.md)*                                             | [`CreativeWork`](./creative-work.md) |
| `abstract`       | A short description that summarizes a `CreativeWork`.                                                                   | [`Block`](./block.md)*                                                            | [`CreativeWork`](./creative-work.md) |
| `authors`        | The authors of the `CreativeWork`.                                                                                      | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `provenance`     | A summary of the provenance of the content within the work.                                                             | [`ProvenanceCount`](./provenance-count.md)*                                       | [`CreativeWork`](./creative-work.md) |
| `contributors`   | A secondary contributor to the `CreativeWork`.                                                                          | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `editors`        | People who edited the `CreativeWork`.                                                                                   | [`Person`](./person.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `maintainers`    | The maintainers of the `CreativeWork`.                                                                                  | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `comments`       | Comments about this creative work.                                                                                      | [`Comment`](./comment.md)*                                                        | [`CreativeWork`](./creative-work.md) |
| `dateCreated`    | Date/time of creation.                                                                                                  | [`DateTime`](./date-time.md)                                                      | [`CreativeWork`](./creative-work.md) |
| `dateReceived`   | Date/time that work was received.                                                                                       | [`DateTime`](./date-time.md)                                                      | [`CreativeWork`](./creative-work.md) |
| `dateAccepted`   | Date/time of acceptance.                                                                                                | [`DateTime`](./date-time.md)                                                      | [`CreativeWork`](./creative-work.md) |
| `dateModified`   | Date/time of most recent modification.                                                                                  | [`DateTime`](./date-time.md)                                                      | [`CreativeWork`](./creative-work.md) |
| `datePublished`  | Date of first publication.                                                                                              | [`DateTime`](./date-time.md)                                                      | [`CreativeWork`](./creative-work.md) |
| `funders`        | People or organizations that funded the `CreativeWork`.                                                                 | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `fundedBy`       | Grants that funded the `CreativeWork`; reverse of `fundedItems`.                                                        | ([`Grant`](./grant.md) \| [`MonetaryGrant`](./monetary-grant.md))*                | [`CreativeWork`](./creative-work.md) |
| `genre`          | Genre of the creative work, broadcast channel or group.                                                                 | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `keywords`       | Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.  | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `isPartOf`       | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [`CreativeWorkVariant`](./creative-work-variant.md)                               | [`CreativeWork`](./creative-work.md) |
| `licenses`       | License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.         | ([`CreativeWorkVariant`](./creative-work-variant.md) \| [`String`](./string.md))* | [`CreativeWork`](./creative-work.md) |
| `parts`          | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [`CreativeWorkVariant`](./creative-work-variant.md)*                              | [`CreativeWork`](./creative-work.md) |
| `publisher`      | A publisher of the CreativeWork.                                                                                        | [`Person`](./person.md) \| [`Organization`](./organization.md)                    | [`CreativeWork`](./creative-work.md) |
| `bibliography`   | A bibliography of references which may be cited in the work.                                                            | [`Bibliography`](./bibliography.md)                                               | [`CreativeWork`](./creative-work.md) |
| `references`     | References to other creative works, such as another publication, web page, scholarly article, etc.                      | [`Reference`](./reference.md)*                                                    | [`CreativeWork`](./creative-work.md) |
| `text`           | The textual content of this creative work.                                                                              | [`Text`](./text.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `title`          | The title of the creative work.                                                                                         | [`Inline`](./inline.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `repository`     | URL of the repository where the un-compiled, human readable source of the work is located.                              | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `path`           | The file system path of the source of the work.                                                                         | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `commit`         | The commit hash (or similar) of the source of the work.                                                                 | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `worktreeStatus` | The status of the source worktree relative to the commit.                                                               | [`WorktreeStatus`](./worktree-status.md)                                          | [`CreativeWork`](./creative-work.md) |
| `version`        | The version of the creative work.                                                                                       | [`String`](./string.md) \| [`Number`](./number.md)                                | [`CreativeWork`](./creative-work.md) |
| `alternateNames` | Alternate names (aliases) for the item.                                                                                 | [`String`](./string.md)*                                                          | [`Thing`](./thing.md)                |
| `description`    | A description of the item.                                                                                              | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `identifiers`    | Any kind of identifier for any kind of Thing.                                                                           | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*              | [`Thing`](./thing.md)                |
| `images`         | Images of the item.                                                                                                     | [`ImageObject`](./image-object.md)*                                               | [`Thing`](./thing.md)                |
| `name`           | The name of the item.                                                                                                   | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `url`            | The URL of the item.                                                                                                    | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `id`             | The identifier for this item.                                                                                           | [`String`](./string.md)                                                           | [`Entity`](./entity.md)              |

# Related

The `ResearchObject` type is related to these types:

- Parents: [`CreativeWork`](./creative-work.md)
- Children: [`Claim`](./claim.md), [`Evidence`](./evidence.md), [`Protocol`](./protocol.md), [`Question`](./question.md), [`Request`](./request.md)

# Bindings

The `ResearchObject` type is represented in:

- [JSON-LD](https://stencila.org/ResearchObject.jsonld)
- [JSON Schema](https://stencila.org/ResearchObject.schema.json)
- Python class [`ResearchObject`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ResearchObject`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/research_object.rs)
- TypeScript class [`ResearchObject`](https://github.com/stencila/stencila/blob/main/ts/src/types/ResearchObject.ts)

***

This documentation was generated from [`ResearchObject.yaml`](https://github.com/stencila/stencila/blob/main/schema/ResearchObject.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
