# Execution Digest

**A digest of the execution state of a node.**

**`@id`**: `stencila:ExecutionDigest`

## Properties

The `ExecutionDigest` type has these properties:

| Name               | Aliases                                  | `@id`                                | Type                                                                                            | Description                                                                            | Inherited from                                                                                   |
| ------------------ | ---------------------------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| id                 | -                                        | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.                                                          | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| stateDigest        | state-digest, state_digest               | `stencila:stateDigest`               | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | A digest of the state of a node.                                                       | -                                                                                                |
| semanticDigest     | semantic-digest, semantic_digest         | `stencila:semanticDigest`            | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | A digest of the "semantic intent" of the resource with respect to the dependency graph | -                                                                                                |
| dependenciesDigest | dependencies-digest, dependencies_digest | `stencila:dependenciesDigest`        | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | A digest of the semantic digests the dependencies of a resource.                       | -                                                                                                |
| dependenciesStale  | dependencies-stale, dependencies_stale   | `stencila:dependenciesStale`         | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | A count of the number of execution dependencies that are stale                         | -                                                                                                |
| dependenciesFailed | dependencies-failed, dependencies_failed | `stencila:dependenciesFailed`        | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | A count of the number of execution dependencies that failed                            | -                                                                                                |

## Related

The `ExecutionDigest` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `ExecutionDigest` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         |                  |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | ⚠️ High loss     |              | 🚧 Under development    |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |       |

## Bindings

The `ExecutionDigest` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ExecutionDigest.jsonld)
- [JSON Schema](https://stencila.dev/ExecutionDigest.schema.json)
- Python class [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_digest.py)
- Rust struct [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_digest.rs)
- TypeScript class [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ExecutionDigest.ts)

## Source

This documentation was generated from [`ExecutionDigest.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDigest.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).