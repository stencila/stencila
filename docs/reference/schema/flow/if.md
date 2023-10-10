# If

**Show and execute alternative content conditional upon an executed expression**

**`@id`**: `stencila:If`

This type is marked as unstable and is subject to change.

## Properties

The `If` type has these properties:

| Name                  | `@id`                                | Type                                                                                                                        | Description                                                          | Inherited from                                                                                          |
| --------------------- | ------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| id                    | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The identifier for this item                                         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)        |
| executionAuto         | `stencila:executionAuto`             | [`ExecutionAuto`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-auto.md)              | Under which circumstances the code should be automatically executed. | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| compilationDigest     | `stencila:compilationDigest`         | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md)          | A digest of the content, semantics and dependencies of the node.     | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDigest       | `stencila:executionDigest`           | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md)          | The `compileDigest` of the node when it was last executed.           | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDependencies | `stencila:executionDependencies`     | [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependency.md)* | The upstream dependencies of this node.                              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDependants   | `stencila:executionDependants`       | [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md)*   | The downstream dependants of this node.                              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionTags         | `stencila:executionTags`             | [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-tag.md)*               | Tags in the code which affect its execution                          | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionCount        | `stencila:executionCount`            | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                           | A count of the number of times that the node has been executed.      | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionRequired     | `stencila:executionRequired`         | [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-required.md)      | Whether, and why, the code requires execution or re-execution.       | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionKernel       | `stencila:executionKernel`           | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The id of the kernel that the node was last executed in.             | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionStatus       | `stencila:executionStatus`           | [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-status.md)          | Status of the most recent, including any current, execution.         | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionEnded        | `stencila:executionEnded`            | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                       | The timestamp when the last execution ended.                         | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDuration     | `stencila:executionDuration`         | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                         | Duration of the last execution.                                      | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| errors                | `stencila:errors`                    | [`CodeError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-error.md)*                     | Errors when compiling (e.g. syntax errors) or executing the node.    | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| clauses               | `stencila:clauses`                   | [`IfClause`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if-clause.md)*                       | The clauses making up the `If` node                                  | [`If`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if.md)                 |

## Related

The `If` type is related to these types:

- Parents: [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)
- Children: none

## Formats

The `If` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                                   |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | --------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    | Encoded to tag [`<div>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         |                  |              | 🚧 Under development    |                                                                                         |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | ⚠️ High loss     |              | 🚧 Under development    | Encoded using special function                                                          |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                         |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                         |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                         |

## Bindings

The `If` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/If.jsonld)
- [JSON Schema](https://stencila.dev/If.schema.json)
- Python class [`If`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/if.py)
- Rust struct [`If`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/if.rs)
- TypeScript class [`If`](https://github.com/stencila/stencila/blob/main/typescript/src/types/If.ts)

## Source

This documentation was generated from [`If.yaml`](https://github.com/stencila/stencila/blob/main/schema/If.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).