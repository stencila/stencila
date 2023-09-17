---
title:
- type: Text
  value: ExecutionDependant
---

# Execution Dependant

**A downstream execution dependant of a node**

**`@id`**: `stencila:ExecutionDependant`

## Properties

The `ExecutionDependant` type has these properties:

| Name              | `@id`                                | Type                                                                                                         | Description                                            | Inherited from                                                                              |
| ----------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| id                | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                           | The identifier for this item                           | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                         |
| dependantRelation | `stencila:dependantRelation`         | [`ExecutionDependantRelation`](https://stencila.dev/docs/reference/schema/flow/execution-dependant-relation) | The relation to the dependant                          | [`ExecutionDependant`](https://stencila.dev/docs/reference/schema/flow/execution-dependant) |
| dependantNode     | `stencila:dependantNode`             | [`ExecutionDependantNode`](https://stencila.dev/docs/reference/schema/flow/execution-dependant-node)         | The node that is the dependant                         | [`ExecutionDependant`](https://stencila.dev/docs/reference/schema/flow/execution-dependant) |
| codeLocation      | `stencila:codeLocation`              | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)*                                        | The location that the dependant is defined within code | [`ExecutionDependant`](https://stencila.dev/docs/reference/schema/flow/execution-dependant) |

## Related

The `ExecutionDependant` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `ExecutionDependant` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ExecutionDependant` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ExecutionDependant.jsonld)
- [JSON Schema](https://stencila.dev/ExecutionDependant.schema.json)
- Python class [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/python/stencila/types/execution_dependant.py)
- Rust struct [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_dependant.rs)
- TypeScript class [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ExecutionDependant.ts)

## Source

This documentation was generated from [`ExecutionDependant.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDependant.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).