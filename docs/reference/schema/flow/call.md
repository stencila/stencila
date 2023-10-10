# Call

**Call another document, optionally with arguments, and include its executed content.**

**`@id`**: `stencila:Call`

This type is marked as unstable and is subject to change.

## Properties

The `Call` type has these properties:

| Name                  | `@id`                                                        | Type                                                                                                                        | Description                                                          | Inherited from                                                                                          |
| --------------------- | ------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| id                    | [`schema:id`](https://schema.org/id)                         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The identifier for this item                                         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)        |
| executionAuto         | `stencila:executionAuto`                                     | [`ExecutionAuto`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-auto.md)              | Under which circumstances the code should be automatically executed. | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| compilationDigest     | `stencila:compilationDigest`                                 | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md)          | A digest of the content, semantics and dependencies of the node.     | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDigest       | `stencila:executionDigest`                                   | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md)          | The `compileDigest` of the node when it was last executed.           | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDependencies | `stencila:executionDependencies`                             | [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependency.md)* | The upstream dependencies of this node.                              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDependants   | `stencila:executionDependants`                               | [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md)*   | The downstream dependants of this node.                              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionTags         | `stencila:executionTags`                                     | [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-tag.md)*               | Tags in the code which affect its execution                          | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionCount        | `stencila:executionCount`                                    | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                           | A count of the number of times that the node has been executed.      | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionRequired     | `stencila:executionRequired`                                 | [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-required.md)      | Whether, and why, the code requires execution or re-execution.       | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionKernel       | `stencila:executionKernel`                                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The id of the kernel that the node was last executed in.             | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionStatus       | `stencila:executionStatus`                                   | [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-status.md)          | Status of the most recent, including any current, execution.         | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionEnded        | `stencila:executionEnded`                                    | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                       | The timestamp when the last execution ended.                         | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDuration     | `stencila:executionDuration`                                 | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                         | Duration of the last execution.                                      | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| errors                | `stencila:errors`                                            | [`CodeError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-error.md)*                     | Errors when compiling (e.g. syntax errors) or executing the node.    | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| source                | `stencila:source`                                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The external source of the content, a file path or URL.              | [`Include`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include.md)       |
| mediaType             | [`schema:encodingFormat`](https://schema.org/encodingFormat) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | Media type of the source content.                                    | [`Include`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include.md)       |
| select                | `stencila:select`                                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | A query to select a subset of content from the source                | [`Include`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include.md)       |
| content               | `stencila:content`                                           | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                             | The structured content decoded from the source.                      | [`Include`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include.md)       |
| arguments             | `stencila:arguments`                                         | [`CallArgument`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call-argument.md)*               | The value of the source document's parameters to call it with        | [`Call`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call.md)             |

## Related

The `Call` type is related to these types:

- Parents: [`Include`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include.md)
- Children: none

## Formats

The `Call` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                          |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ------------------------------ |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    |                                |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         |                  |              | 🚧 Under development    |                                |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | ⚠️ High loss     |              | 🚧 Under development    | Encoded using special function |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                |

## Bindings

The `Call` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Call.jsonld)
- [JSON Schema](https://stencila.dev/Call.schema.json)
- Python class [`Call`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/call.py)
- Rust struct [`Call`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/call.rs)
- TypeScript class [`Call`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Call.ts)

## Source

This documentation was generated from [`Call.yaml`](https://github.com/stencila/stencila/blob/main/schema/Call.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).