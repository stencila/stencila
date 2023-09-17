---
title:
- type: Text
  value: Enumeration
---

# Enumeration

**Lists or enumerations, for example, a list of cuisines or music genres, etc.**

**`@id`**: [`schema:Enumeration`](https://schema.org/Enumeration)

## Properties

The `Enumeration` type has these properties:

| Name           | `@id`                                                      | Type                                                                                                                                                       | Description                                   | Inherited from                                                      |
| -------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------- |
| id             | [`schema:id`](https://schema.org/id)                       | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                  | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| alternateNames | [`schema:alternateName`](https://schema.org/alternateName) | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.       | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| description    | [`schema:description`](https://schema.org/description)     | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                    | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| identifiers    | [`schema:identifier`](https://schema.org/identifier)       | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing. | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| images         | [`schema:image`](https://schema.org/image)                 | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                           | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| name           | [`schema:name`](https://schema.org/name)                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                         | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |
| url            | [`schema:url`](https://schema.org/url)                     | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                          | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)   |

## Related

The `Enumeration` type is related to these types:

- Parents: [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)
- Children: [`CitationIntent`](https://stencila.dev/docs/reference/schema/prose/citation-intent), [`CitationMode`](https://stencila.dev/docs/reference/schema/prose/citation-mode), [`ClaimType`](https://stencila.dev/docs/reference/schema/works/claim-type), [`ExecutionAuto`](https://stencila.dev/docs/reference/schema/flow/execution-auto), [`ExecutionDependantRelation`](https://stencila.dev/docs/reference/schema/flow/execution-dependant-relation), [`ExecutionDependencyRelation`](https://stencila.dev/docs/reference/schema/flow/execution-dependency-relation), [`ExecutionRequired`](https://stencila.dev/docs/reference/schema/flow/execution-required), [`ExecutionStatus`](https://stencila.dev/docs/reference/schema/flow/execution-status), [`FormDeriveAction`](https://stencila.dev/docs/reference/schema/flow/form-derive-action), [`ListOrder`](https://stencila.dev/docs/reference/schema/prose/list-order), [`NoteType`](https://stencila.dev/docs/reference/schema/prose/note-type), [`TableCellType`](https://stencila.dev/docs/reference/schema/works/table-cell-type), [`TableRowType`](https://stencila.dev/docs/reference/schema/works/table-row-type), [`TimeUnit`](https://stencila.dev/docs/reference/schema/data/time-unit)

## Formats

The `Enumeration` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Enumeration` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Enumeration.jsonld)
- [JSON Schema](https://stencila.dev/Enumeration.schema.json)
- Python class [`Enumeration`](https://github.com/stencila/stencila/blob/main/python/stencila/types/enumeration.py)
- Rust struct [`Enumeration`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/enumeration.rs)
- TypeScript class [`Enumeration`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Enumeration.ts)

## Source

This documentation was generated from [`Enumeration.yaml`](https://github.com/stencila/stencila/blob/main/schema/Enumeration.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).