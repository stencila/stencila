---
title:
- type: Text
  value: ExecutionDigest
---

# Execution Digest

**A digest of the execution state of a node.**

**`@id`**: `stencila:ExecutionDigest`

## Properties

The `ExecutionDigest` type has these properties:

| Name               | `@id`                                | Type                                                               | Description                                                                            | Inherited from                                                                        |
| ------------------ | ------------------------------------ | ------------------------------------------------------------------ | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| id                 | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item                                                           | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                   |
| stateDigest        | `stencila:stateDigest`               | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | A digest of the state of a node.                                                       | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) |
| semanticDigest     | `stencila:semanticDigest`            | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | A digest of the "semantic intent" of the resource with respect to the dependency graph | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) |
| dependenciesDigest | `stencila:dependenciesDigest`        | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | A digest of the semantic digests the dependencies of a resource.                       | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) |
| dependenciesStale  | `stencila:dependenciesStale`         | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | A count of the number of execution dependencies that are stale                         | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) |
| dependenciesFailed | `stencila:dependenciesFailed`        | [`Number`](https://stencila.dev/docs/reference/schema/data/number) | A count of the number of execution dependencies that failed                            | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest) |

## Related

The `ExecutionDigest` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `ExecutionDigest` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ExecutionDigest` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ExecutionDigest.jsonld)
- [JSON Schema](https://stencila.dev/ExecutionDigest.schema.json)
- Python class [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/python/stencila/types/execution_digest.py)
- Rust struct [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_digest.rs)
- TypeScript class [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ExecutionDigest.ts)

## Source

This documentation was generated from [`ExecutionDigest.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDigest.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).