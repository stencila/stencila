---
title:
- type: Text
  value: ExecutionDependency
---

# Execution Dependency

**An upstream execution dependency of a node**

**`@id`**: `stencila:ExecutionDependency`

## Properties

The `ExecutionDependency` type has these properties:

| Name               | `@id`                                | Type                                                                                                           | Description                                             | Inherited from                                                                                |
| ------------------ | ------------------------------------ | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| id                 | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                             | The identifier for this item                            | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                           |
| dependencyRelation | `stencila:dependencyRelation`        | [`ExecutionDependencyRelation`](https://stencila.dev/docs/reference/schema/flow/execution-dependency-relation) | The relation to the dependency                          | [`ExecutionDependency`](https://stencila.dev/docs/reference/schema/flow/execution-dependency) |
| dependencyNode     | `stencila:dependencyNode`            | [`ExecutionDependencyNode`](https://stencila.dev/docs/reference/schema/flow/execution-dependency-node)         | The node that is the dependency                         | [`ExecutionDependency`](https://stencila.dev/docs/reference/schema/flow/execution-dependency) |
| codeLocation       | `stencila:codeLocation`              | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)*                                          | The location that the dependency is defined within code | [`ExecutionDependency`](https://stencila.dev/docs/reference/schema/flow/execution-dependency) |

## Related

The `ExecutionDependency` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `ExecutionDependency` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `ExecutionDependency` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ExecutionDependency.jsonld)
- [JSON Schema](https://stencila.dev/ExecutionDependency.schema.json)
- Python class [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/python/stencila/types/execution_dependency.py)
- Rust struct [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_dependency.rs)
- TypeScript class [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ExecutionDependency.ts)

## Source

This documentation was generated from [`ExecutionDependency.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDependency.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).