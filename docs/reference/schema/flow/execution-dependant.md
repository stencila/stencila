# Execution Dependant

**A downstream execution dependant of a node**

**`@id`**: `stencila:ExecutionDependant`

## Properties

The `ExecutionDependant` type has these properties:

| Name              | `@id`                                | Type                                                                                                                                      | Description                                            | Inherited from                                                                                                           |
| ----------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| id                | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                           | The identifier for this item                           | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)                         |
| dependantRelation | `stencila:dependantRelation`         | [`ExecutionDependantRelation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant-relation.md) | The relation to the dependant                          | [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md) |
| dependantNode     | `stencila:dependantNode`             | [`ExecutionDependantNode`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant-node.md)         | The node that is the dependant                         | [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md) |
| codeLocation      | `stencila:codeLocation`              | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)*                                        | The location that the dependant is defined within code | [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md) |

## Related

The `ExecutionDependant` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `ExecutionDependant` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                            | Encoding       | Decoding     | Status                 | Notes |
| ------------------------------------------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/HTML.md)             | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JATS.md)             | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Markdown.md)     | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Plain text.md) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON5.md)           | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/YAML.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Debug.md)           | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `ExecutionDependant` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ExecutionDependant.jsonld)
- [JSON Schema](https://stencila.dev/ExecutionDependant.schema.json)
- Python class [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/python/stencila/types/execution_dependant.py)
- Rust struct [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_dependant.rs)
- TypeScript class [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ExecutionDependant.ts)

## Source

This documentation was generated from [`ExecutionDependant.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDependant.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).