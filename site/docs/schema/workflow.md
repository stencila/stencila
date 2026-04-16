---
title: Workflow
description: An AI workflow definition.
---

This is a type used in Stencila Schema for reusable AI workflow definitions.

It exists to represent a workflow as a document-like artifact with metadata,
human-readable guidance, and a machine-executable pipeline defined in Graphviz
DOT. Workflows are discovered from `.stencila/workflows/<name>/WORKFLOW.md`
and can reference agents by name, allowing shared workflow structures to be
combined with workspace or user-level agent definitions.

Key properties include `name`, `content`, `pipeline`, `goal`, `goalHint`, and
`overrides`.


# Properties

The `Workflow` type has these properties:

| Name                  | Description                                                                                                             | Type                                                                              | Inherited from                       |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | ------------------------------------ |
| `name`                | The name of the workflow.                                                                                               | [`String`](./string.md)                                                           | -                                    |
| `whenToUse`           | Positive selection signals describing when this workflow should be used.                                                | [`String`](./string.md)*                                                          | -                                    |
| `whenNotToUse`        | Negative selection signals describing when this workflow should not be used.                                            | [`String`](./string.md)*                                                          | -                                    |
| `frontmatter`         | Frontmatter containing workflow metadata.                                                                               | [`String`](./string.md)                                                           | -                                    |
| `content`             | The content of the workflow (Markdown body containing the DOT pipeline and documentation).                              | [`Block`](./block.md)*                                                            | -                                    |
| `pipeline`            | The raw DOT source defining the pipeline digraph.                                                                       | [`String`](./string.md)                                                           | -                                    |
| `goal`                | A fixed, predetermined high-level goal for the pipeline.                                                                | [`String`](./string.md)                                                           | -                                    |
| `goalHint`            | Hint text displayed in user interfaces to guide the user to provide a specific goal.                                    | [`String`](./string.md)                                                           | -                                    |
| `overrides`           | CSS-like rules for per-node agent overrides across the pipeline.                                                        | [`String`](./string.md)                                                           | -                                    |
| `defaultMaxRetry`     | Global retry ceiling for nodes that omit max_retries.                                                                   | [`Integer`](./integer.md)                                                         | -                                    |
| `retryTarget`         | Node ID to jump to if exit is reached with unsatisfied goal gates.                                                      | [`String`](./string.md)                                                           | -                                    |
| `fallbackRetryTarget` | Secondary jump target if retryTarget is missing or invalid.                                                             | [`String`](./string.md)                                                           | -                                    |
| `defaultFidelity`     | Default context fidelity mode for LLM sessions.                                                                         | [`String`](./string.md)                                                           | -                                    |
| `workType`            | The type of `CreativeWork` (e.g. article, book, software application).                                                  | [`CreativeWorkType`](./creative-work-type.md)                                     | [`CreativeWork`](./creative-work.md) |
| `doi`                 | The work's Digital Object Identifier (https://doi.org/).                                                                | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `about`               | The subject matter of the content.                                                                                      | [`ThingVariant`](./thing-variant.md)*                                             | [`CreativeWork`](./creative-work.md) |
| `abstract`            | A short description that summarizes a `CreativeWork`.                                                                   | [`Block`](./block.md)*                                                            | [`CreativeWork`](./creative-work.md) |
| `authors`             | The authors of the `CreativeWork`.                                                                                      | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `provenance`          | A summary of the provenance of the content within the work.                                                             | [`ProvenanceCount`](./provenance-count.md)*                                       | [`CreativeWork`](./creative-work.md) |
| `contributors`        | A secondary contributor to the `CreativeWork`.                                                                          | [`Author`](./author.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `editors`             | People who edited the `CreativeWork`.                                                                                   | [`Person`](./person.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `maintainers`         | The maintainers of the `CreativeWork`.                                                                                  | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `comments`            | Comments about this creative work.                                                                                      | [`Comment`](./comment.md)*                                                        | [`CreativeWork`](./creative-work.md) |
| `dateCreated`         | Date/time of creation.                                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateReceived`        | Date/time that work was received.                                                                                       | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateAccepted`        | Date/time of acceptance.                                                                                                | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `dateModified`        | Date/time of most recent modification.                                                                                  | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `datePublished`       | Date of first publication.                                                                                              | [`Date`](./date.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `funders`             | People or organizations that funded the `CreativeWork`.                                                                 | ([`Person`](./person.md) \| [`Organization`](./organization.md))*                 | [`CreativeWork`](./creative-work.md) |
| `fundedBy`            | Grants that funded the `CreativeWork`; reverse of `fundedItems`.                                                        | ([`Grant`](./grant.md) \| [`MonetaryGrant`](./monetary-grant.md))*                | [`CreativeWork`](./creative-work.md) |
| `genre`               | Genre of the creative work, broadcast channel or group.                                                                 | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `keywords`            | Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.  | [`String`](./string.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `isPartOf`            | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [`CreativeWorkVariant`](./creative-work-variant.md)                               | [`CreativeWork`](./creative-work.md) |
| `licenses`            | License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.         | ([`CreativeWorkVariant`](./creative-work-variant.md) \| [`String`](./string.md))* | [`CreativeWork`](./creative-work.md) |
| `parts`               | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [`CreativeWorkVariant`](./creative-work-variant.md)*                              | [`CreativeWork`](./creative-work.md) |
| `publisher`           | A publisher of the CreativeWork.                                                                                        | [`Person`](./person.md) \| [`Organization`](./organization.md)                    | [`CreativeWork`](./creative-work.md) |
| `bibliography`        | A bibliography of references which may be cited in the work.                                                            | [`Bibliography`](./bibliography.md)                                               | [`CreativeWork`](./creative-work.md) |
| `references`          | References to other creative works, such as another publication, web page, scholarly article, etc.                      | [`Reference`](./reference.md)*                                                    | [`CreativeWork`](./creative-work.md) |
| `text`                | The textual content of this creative work.                                                                              | [`Text`](./text.md)                                                               | [`CreativeWork`](./creative-work.md) |
| `title`               | The title of the creative work.                                                                                         | [`Inline`](./inline.md)*                                                          | [`CreativeWork`](./creative-work.md) |
| `repository`          | URL of the repository where the un-compiled, human readable source of the work is located.                              | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `path`                | The file system path of the source of the work.                                                                         | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `commit`              | The commit hash (or similar) of the source of the work.                                                                 | [`String`](./string.md)                                                           | [`CreativeWork`](./creative-work.md) |
| `version`             | The version of the creative work.                                                                                       | [`String`](./string.md) \| [`Number`](./number.md)                                | [`CreativeWork`](./creative-work.md) |
| `alternateNames`      | Alternate names (aliases) for the item.                                                                                 | [`String`](./string.md)*                                                          | [`Thing`](./thing.md)                |
| `description`         | A description of the item.                                                                                              | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `identifiers`         | Any kind of identifier for any kind of Thing.                                                                           | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*              | [`Thing`](./thing.md)                |
| `images`              | Images of the item.                                                                                                     | [`ImageObject`](./image-object.md)*                                               | [`Thing`](./thing.md)                |
| `url`                 | The URL of the item.                                                                                                    | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `id`                  | The identifier for this item.                                                                                           | [`String`](./string.md)                                                           | [`Entity`](./entity.md)              |

# Related

The `Workflow` type is related to these types:

- Parents: [`CreativeWork`](./creative-work.md)
- Children: none

# Bindings

The `Workflow` type is represented in:

- [JSON-LD](https://stencila.org/Workflow.jsonld)
- [JSON Schema](https://stencila.org/Workflow.schema.json)
- Python class [`Workflow`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Workflow`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/workflow.rs)
- TypeScript class [`Workflow`](https://github.com/stencila/stencila/blob/main/ts/src/types/Workflow.ts)

***

This documentation was generated from [`Workflow.yaml`](https://github.com/stencila/stencila/blob/main/schema/Workflow.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
