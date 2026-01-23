---
title: Prompt
description: A prompt for creating or editing document content.
---

# Properties

The `Prompt` type has these properties:

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
| `frontmatter`           | Frontmatter containing document metadata.                                                                               | [`String`](./string.md)                                                           | -                                    |
| `instructionTypes`      | The types of instructions that the prompt supports                                                                      | [`InstructionType`](./instruction-type.md)*                                       | -                                    |
| `nodeTypes`             | The types of nodes that the prompt supports                                                                             | [`String`](./string.md)*                                                          | -                                    |
| `nodeCount`             | The number of nodes that the prompt supports                                                                            | [`UnsignedInteger`](./unsigned-integer.md) \| [`String`](./string.md)             | -                                    |
| `queryPatterns`         | Regular expressions used to match the prompt with a user query                                                          | [`String`](./string.md)*                                                          | -                                    |
| `content`               | The content of the prompt.                                                                                              | [`Block`](./block.md)*                                                            | -                                    |

# Related

The `Prompt` type is related to these types:

- Parents: [`CreativeWork`](./creative-work.md)[`Executable`](./executable.md)
- Children: none

# Bindings

The `Prompt` type is represented in:

- [JSON-LD](https://stencila.org/Prompt.jsonld)
- [JSON Schema](https://stencila.org/Prompt.schema.json)
- Python class [`Prompt`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Prompt`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/prompt.rs)
- TypeScript class [`Prompt`](https://github.com/stencila/stencila/blob/main/ts/src/types/Prompt.ts)

***

This documentation was generated from [`Prompt.yaml`](https://github.com/stencila/stencila/blob/main/schema/Prompt.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
