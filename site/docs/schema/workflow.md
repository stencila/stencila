---
title: Workflow
description: A workflow pipeline definition using Graphviz DOT syntax to orchestrate multi-stage AI tasks.
---

A workflow is a directory containing a `WORKFLOW.md` file with YAML frontmatter and a Markdown
body. The pipeline is defined as a Graphviz DOT digraph in the first ```dot code block in the
body. Additional Markdown content provides human-readable documentation for the workflow.

Workflows are discovered from `.stencila/workflows/<name>/WORKFLOW.md`.
Each node in the DOT graph can reference an agent by name via the `agent` attribute; agents are
resolved from workspace `.stencila/agents/` and user-level `~/.config/stencila/agents/`.
Because workflows are typically shared (committed to a repository as part of a lab), while
agents may be personal (user-level agents can encapsulate individual model preferences and
provider configuration), this separation allows the same shared workflow to be executed with
different agent configurations by different users. Agent definitions provide defaults for
model, provider, system instructions, and tools, but explicit node attributes override them.

The DOT graph follows the Attractor specification: a strict subset of digraph syntax with typed
attributes for node handlers, edge routing, retry policies, and human-in-the-loop gates.


# Properties

The `Workflow` type has these properties:

| Name                  | Description                                                                                                             | Type                                                                              | Inherited from                       |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | ------------------------------------ |
| `id`                  | The identifier for this item.                                                                                           | [`String`](./string.md)                                                           | [`Entity`](./entity.md)              |
| `alternateNames`      | Alternate names (aliases) for the item.                                                                                 | [`String`](./string.md)*                                                          | [`Thing`](./thing.md)                |
| `description`         | A description of the item.                                                                                              | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
| `identifiers`         | Any kind of identifier for any kind of Thing.                                                                           | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*              | [`Thing`](./thing.md)                |
| `images`              | Images of the item.                                                                                                     | [`ImageObject`](./image-object.md)*                                               | [`Thing`](./thing.md)                |
| `name`                | The name of the workflow.                                                                                               | [`String`](./string.md)                                                           | -                                    |
| `url`                 | The URL of the item.                                                                                                    | [`String`](./string.md)                                                           | [`Thing`](./thing.md)                |
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
| `frontmatter`         | Frontmatter containing workflow metadata.                                                                               | [`String`](./string.md)                                                           | -                                    |
| `content`             | The content of the workflow (Markdown body containing the DOT pipeline and documentation).                              | [`Block`](./block.md)*                                                            | -                                    |
| `pipeline`            | The raw DOT source defining the pipeline digraph.                                                                       | [`String`](./string.md)                                                           | -                                    |
| `goal`                | The high-level goal for the pipeline.                                                                                   | [`String`](./string.md)                                                           | -                                    |
| `overrides`           | CSS-like rules for per-node agent overrides across the pipeline.                                                        | [`String`](./string.md)                                                           | -                                    |
| `defaultMaxRetry`     | Global retry ceiling for nodes that omit max_retries.                                                                   | [`Integer`](./integer.md)                                                         | -                                    |
| `retryTarget`         | Node ID to jump to if exit is reached with unsatisfied goal gates.                                                      | [`String`](./string.md)                                                           | -                                    |
| `fallbackRetryTarget` | Secondary jump target if retryTarget is missing or invalid.                                                             | [`String`](./string.md)                                                           | -                                    |
| `defaultFidelity`     | Default context fidelity mode for LLM sessions.                                                                         | [`String`](./string.md)                                                           | -                                    |

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
