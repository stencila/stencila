---
title: Article
description: An article, including news and scholarly articles.
---

This is an implementation, and extension, of schema.org [`Article`](https://schema.org/Article).
It extends schema.org `Article` by adding a `content` property which must be
an array of [`Block`](./block.md), as well as the properties added by
[`CreativeWork`](./creative-work.md) which it extends.


# Properties

The `Article` type has these properties:

| Name                    | Description                                                                                                             | Type                                                                              | Inherited from                       |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | ------------------------------------ |
| `id`                    | The identifier for this item.                                                                                           | [`String`](./string.md)                                                           | [`Entity`](./entity.md)              |
| `alternateNames`        | Alternate names (aliases) for the item.                                                                                 | [`String`](./string.md)*                                                          | [`Thing`](./thing.md)                |
| `description`           | A description of the item.                                                                                              | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `identifiers`           | Any kind of identifier for any kind of Thing.                                                                           | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*              | [`Thing`](./thing.md)                |
| `images`                | Images of the item.                                                                                                     | [`ImageObject`](./image-object.md)*                                               | [`Thing`](./thing.md)                |
| `name`                  | The name of the item.                                                                                                   | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `url`                   | The URL of the item.                                                                                                    | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `workType`              | The type of `CreativeWork` (e.g. article, book, software application).                                                  | [`CreativeWorkType`](./creative-work-type.md)                                     | [`CreativeWork`](./creative-work.md) |
| `doi`                   | The work's Digital Object Identifier (https://doi.org/).                                                                | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `about`                 | The subject matter of the content.                                                                                      | [`ThingVariant`](./thing-variant.md)*                                             | [`CreativeWork`](./creative-work.md) |
| `abstract`              | A short description that summarizes a `CreativeWork`.                                                                   | [`Block`](./block.md)*                                                            | [`CreativeWork`](./creative-work.md) |
| `authors`               | The authors of the `CreativeWork`.                                                                                      | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `provenance`            | A summary of the provenance of the content within the work.                                                             | [`ProvenanceCount`](./provenance-count.md)*                                       | [`CreativeWork`](./creative-work.md) |
| `contributors`          | A secondary contributor to the `CreativeWork`.                                                                          | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `editors`               | People who edited the `CreativeWork`.                                                                                   | [`Person`](./person.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `maintainers`           | The maintainers of the `CreativeWork`.                                                                                  | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `comments`              | Comments about this creative work.                                                                                      | [`Comment`](./comment.md)*                                                        | [`CreativeWork`](./creative-work.md) |
| `dateCreated`           | Date/time of creation.                                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateReceived`          | Date/time that work was received.                                                                                       | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateAccepted`          | Date/time of acceptance.                                                                                                | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateModified`          | Date/time of most recent modification.                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `datePublished`         | Date of first publication.                                                                                              | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `funders`               | People or organizations that funded the `CreativeWork`.                                                                 | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `fundedBy`              | Grants that funded the `CreativeWork`; reverse of `fundedItems`.                                                        | ([`Grant`](./grant.md) \| [`MonetaryGrant`](./monetary-grant.md))*                | [`CreativeWork`](./creative-work.md) |
| `genre`                 | Genre of the creative work, broadcast channel or group.                                                                 | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `keywords`              | Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.  | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `isPartOf`              | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [`CreativeWorkVariant`](./creative-work-variant.md)                               | [`CreativeWork`](./creative-work.md) |
| `licenses`              | License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.         | ([`CreativeWorkVariant`](./creative-work-variant.md) \| [`String`](./string.md))* | [`CreativeWork`](./creative-work.md) |
| `parts`                 | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [`CreativeWorkVariant`](./creative-work-variant.md)*                              | [`CreativeWork`](./creative-work.md) |
| `publisher`             | A publisher of the CreativeWork.                                                                                        | [`Person`](./person.md) \| [`Organization`](./organization.md)                    | [`CreativeWork`](./creative-work.md) |
| `bibliography`          | A bibliography of references which may be cited in the work.                                                            | [`Bibliography`](./bibliography.md)                                               | [`CreativeWork`](./creative-work.md) |
| `references`            | References to other creative works, such as another publication, web page, scholarly article, etc.                      | [`Reference`](./reference.md)*                                                    | [`CreativeWork`](./creative-work.md) |
| `text`                  | The textual content of this creative work.                                                                              | [`Text`](./text.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `title`                 | The title of the creative work.                                                                                         | [`Inline`](./inline.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `repository`            | URL of the repository where the un-compiled, human readable source of the work is located.                              | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `path`                  | The file system path of the source of the work.                                                                         | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `commit`                | The commit hash (or similar) of the source of the work.                                                                 | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `version`               | The version of the creative work.                                                                                       | [`String`](./string.md) \| [`Number`](./number.md)                                | [`CreativeWork`](./creative-work.md) |
| `executionMode`         | Under which circumstances the node should be executed.                                                                  | [`ExecutionMode`](./execution-mode.md)                                            | [`Executable`](./executable.md)      |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.                                                        | [`CompilationDigest`](./compilation-digest.md)                                    | [`Executable`](./executable.md)      |
| `compilationMessages`   | Messages generated while compiling the code.                                                                            | [`CompilationMessage`](./compilation-message.md)*                                 | [`Executable`](./executable.md)      |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.                                                          | [`CompilationDigest`](./compilation-digest.md)                                    | [`Executable`](./executable.md)      |
| `executionDependencies` | The upstream dependencies of this node.                                                                                 | [`ExecutionDependency`](./execution-dependency.md)*                               | [`Executable`](./executable.md)      |
| `executionDependants`   | The downstream dependants of this node.                                                                                 | [`ExecutionDependant`](./execution-dependant.md)*                                 | [`Executable`](./executable.md)      |
| `executionTags`         | Tags in the code which affect its execution.                                                                            | [`ExecutionTag`](./execution-tag.md)*                                             | [`Executable`](./executable.md)      |
| `executionCount`        | A count of the number of times that the node has been executed.                                                         | [`Integer`](./integer.md)                                                         | [`Executable`](./executable.md)      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.                                                          | [`ExecutionRequired`](./execution-required.md)                                    | [`Executable`](./executable.md)      |
| `executionStatus`       | Status of the most recent, including any current, execution.                                                            | [`ExecutionStatus`](./execution-status.md)                                        | [`Executable`](./executable.md)      |
| `executionInstance`     | The id of the kernel instance that performed the last execution.                                                        | [`String`](./string.md)                                                           | [`Executable`](./executable.md)      |
| `executionEnded`        | The timestamp when the last execution ended.                                                                            | [`Timestamp`](./timestamp.md)                                                     | [`Executable`](./executable.md)      |
| `executionDuration`     | Duration of the last execution.                                                                                         | [`Duration`](./duration.md)                                                       | [`Executable`](./executable.md)      |
| `executionMessages`     | Messages emitted while executing the node.                                                                              | [`ExecutionMessage`](./execution-message.md)*                                     | [`Executable`](./executable.md)      |
| `pageStart`             | The page on which the article starts; for example "135" or "xiii".                                                      | [`Integer`](./integer.md) \| [`String`](./string.md)                              | -                                    |
| `pageEnd`               | The page on which the article ends; for example "138" or "xvi".                                                         | [`Integer`](./integer.md) \| [`String`](./string.md)                              | -                                    |
| `pagination`            | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".                   | [`String`](./string.md)                                                           | -                                    |
| `frontmatter`           | Frontmatter containing document metadata.                                                                               | [`String`](./string.md)                                                           | -                                    |
| `config`                | Configuration options for the document.                                                                                 | [`Config`](./config.md)                                                           | -                                    |
| `headings`              | A list of links to headings, including implied section headings, within the document                                    | [`List`](./list.md)                                                               | -                                    |
| `content`               | The content of the article.                                                                                             | [`Block`](./block.md)*                                                            | -                                    |
| `archive`               | Nodes, usually from within `content` of the article, that have been archived.                                           | [`Node`](./node.md)*                                                              | -                                    |
| `extra`                 | Additional metadata for the article.                                                                                    | [`Object`](./object.md)                                                           | -                                    |

# Related

The `Article` type is related to these types:

- Parents: [`CreativeWork`](./creative-work.md)[`Executable`](./executable.md)
- Children: none

# Bindings

The `Article` type is represented in:

- [JSON-LD](https://stencila.org/Article.jsonld)
- [JSON Schema](https://stencila.org/Article.schema.json)
- Python class [`Article`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Article`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/article.rs)
- TypeScript class [`Article`](https://github.com/stencila/stencila/blob/main/ts/src/types/Article.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Article` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                | Strategy        |
| --------- | ---------- | ------------------------------------------ | --------------- |
| `content` | Min+       | Generate a single arbitrary block node     | `vec_blocks(1)` |
|           | Low+       | Generate up to two arbitrary block nodes   | `vec_blocks(2)` |
|           | High+      | Generate up to four arbitrary block nodes  | `vec_blocks(4)` |
|           | Max        | Generate up to eight arbitrary block nodes | `vec_blocks(8)` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Article.yaml`](https://github.com/stencila/stencila/blob/main/schema/Article.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
