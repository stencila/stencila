---
title: Reference
description: A reference to a creative work, including books, movies, photographs, software programs, etc.
---

# Properties

The `Reference` type has these properties:

| Name           | Description                                                                                           | Type                                                                 | Inherited from          |
| -------------- | ----------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`           | The identifier for this item.                                                                         | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `workType`     | The type of `CreativeWork` being referenced (e.g. Article, Book, Dataset).                            | [`CreativeWorkType`](./creative-work-type.md)                        | -                       |
| `doi`          | The Digital Object Identifier (https://doi.org/) of the work being referenced.                        | [`String`](./string.md)                                              | -                       |
| `authors`      | The authors of the work.                                                                              | [`Author`](./author.md)*                                             | -                       |
| `editors`      | People who edited the referenced work.                                                                | [`Person`](./person.md)*                                             | -                       |
| `publisher`    | A publisher of the referenced work.                                                                   | [`Person`](./person.md) \| [`Organization`](./organization.md)       | -                       |
| `date`         | Date of first publication.                                                                            | [`Date`](./date.md)                                                  | -                       |
| `title`        | The title of the referenced work.                                                                     | [`Inline`](./inline.md)*                                             | -                       |
| `isPartOf`     | Another `Reference` that this reference is a part of.                                                 | [`Reference`](./reference.md)                                        | -                       |
| `volumeNumber` | Identifies the volume of publication or multi-part work; for example, "iii" or "2".                   | [`Integer`](./integer.md) \| [`String`](./string.md)                 | -                       |
| `issueNumber`  | Identifies the issue of a serial publication; for example, "3" or "12".                               | [`Integer`](./integer.md) \| [`String`](./string.md)                 | -                       |
| `pageStart`    | The page on which the article starts; for example "135" or "xiii".                                    | [`Integer`](./integer.md) \| [`String`](./string.md)                 | -                       |
| `pageEnd`      | The page on which the article ends; for example "138" or "xvi".                                       | [`Integer`](./integer.md) \| [`String`](./string.md)                 | -                       |
| `pagination`   | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55". | [`String`](./string.md)                                              | -                       |
| `version`      | The version/edition of the referenced work.                                                           | [`String`](./string.md) \| [`Number`](./number.md)                   | -                       |
| `identifiers`  | Any kind of identifier for the referenced work.                                                       | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | -                       |
| `url`          | The URL of the referenced work.                                                                       | [`String`](./string.md)                                              | -                       |
| `text`         | Plain text representation of the referenced work.                                                     | [`String`](./string.md)                                              | -                       |
| `content`      | A rendering of the reference using the citation style of the document.                                | [`Inline`](./inline.md)*                                             | -                       |

# Related

The `Reference` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Reference` type is represented in:

- [JSON-LD](https://stencila.org/Reference.jsonld)
- [JSON Schema](https://stencila.org/Reference.schema.json)
- Python class [`Reference`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/reference.py)
- Rust struct [`Reference`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/reference.rs)
- TypeScript class [`Reference`](https://github.com/stencila/stencila/blob/main/ts/src/types/Reference.ts)

# Source

This documentation was generated from [`Reference.yaml`](https://github.com/stencila/stencila/blob/main/schema/Reference.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
