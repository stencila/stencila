---
title: Enumeration
description: Lists or enumerations, for example, a list of cuisines or music genres, etc.
---

# Properties

The `Enumeration` type has these properties:

| Name             | Description                                   | Type                                                                 | Inherited from          |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                           | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                         | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |

# Related

The `Enumeration` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: [`AdmonitionType`](./admonition-type.md), [`AuthorRoleName`](./author-role-name.md), [`CitationIntent`](./citation-intent.md), [`CitationMode`](./citation-mode.md), [`ClaimType`](./claim-type.md), [`ConfigPublishGhostState`](./config-publish-ghost-state.md), [`ConfigPublishGhostType`](./config-publish-ghost-type.md), [`ConfigPublishZenodoAccessRight`](./config-publish-zenodo-access-right.md), [`CreativeWorkType`](./creative-work-type.md), [`ExecutionBounds`](./execution-bounds.md), [`ExecutionDependantRelation`](./execution-dependant-relation.md), [`ExecutionDependencyRelation`](./execution-dependency-relation.md), [`ExecutionMode`](./execution-mode.md), [`ExecutionRequired`](./execution-required.md), [`ExecutionStatus`](./execution-status.md), [`FormDeriveAction`](./form-derive-action.md), [`HorizontalAlignment`](./horizontal-alignment.md), [`InstructionType`](./instruction-type.md), [`LabelType`](./label-type.md), [`ListOrder`](./list-order.md), [`MessageLevel`](./message-level.md), [`MessageRole`](./message-role.md), [`NoteType`](./note-type.md), [`ProvenanceCategory`](./provenance-category.md), [`RelativePosition`](./relative-position.md), [`SectionType`](./section-type.md), [`SuggestionStatus`](./suggestion-status.md), [`TableCellType`](./table-cell-type.md), [`TableRowType`](./table-row-type.md), [`TimeUnit`](./time-unit.md), [`VerticalAlignment`](./vertical-alignment.md)

# Bindings

The `Enumeration` type is represented in:

- [JSON-LD](https://stencila.org/Enumeration.jsonld)
- [JSON Schema](https://stencila.org/Enumeration.schema.json)
- Python class [`Enumeration`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Enumeration`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/enumeration.rs)
- TypeScript class [`Enumeration`](https://github.com/stencila/stencila/blob/main/ts/src/types/Enumeration.ts)

***

This documentation was generated from [`Enumeration.yaml`](https://github.com/stencila/stencila/blob/main/schema/Enumeration.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
