# Instruction Block

**An instruction to edit some block content.**

**`@id`**: `stencila:InstructionBlock`

## Properties

The `InstructionBlock` type has these properties:

| Name                    | Aliases                                                                                                                   | `@id`                                        | Type                                                                                                                         | Description                                                          | Inherited from                                                                                             |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `id`                    | -                                                                                                                         | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                              | The identifier for this item.                                        | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)           |
| `autoExec`              | `auto`, `auto-exec`, `auto_exec`                                                                                          | `stencila:autoExec`                          | [`AutomaticExecution`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/automatic-execution.md)     | Under which circumstances the code should be automatically executed. | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `compilationDigest`     | `compilation-digest`, `compilation_digest`                                                                                | `stencila:compilationDigest`                 | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)       | A digest of the content, semantics and dependencies of the node.     | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `compilationErrors`     | `compilation-errors`, `compilation_errors`, `compilationError`, `compilation-error`, `compilation_error`                  | `stencila:compilationErrors`                 | [`CompilationError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-error.md)*        | Errors generated when compiling the code.                            | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionDigest`       | `execution-digest`, `execution_digest`                                                                                    | `stencila:executionDigest`                   | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)       | The `compilationDigest` of the node when it was last executed.       | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionDependencies` | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` | `stencila:executionDependencies`             | [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependency.md)*  | The upstream dependencies of this node.                              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionDependants`   | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        | `stencila:executionDependants`               | [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md)*    | The downstream dependants of this node.                              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionTags`         | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      | `stencila:executionTags`                     | [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-tag.md)*                | Tags in the code which affect its execution.                         | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionCount`        | `execution-count`, `execution_count`                                                                                      | `stencila:executionCount`                    | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                            | A count of the number of times that the node has been executed.      | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionRequired`     | `execution-required`, `execution_required`                                                                                | `stencila:executionRequired`                 | [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-required.md)       | Whether, and why, the code requires execution or re-execution.       | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionStatus`       | `execution-status`, `execution_status`                                                                                    | `stencila:executionStatus`                   | [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-status.md)           | Status of the most recent, including any current, execution.         | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionActor`        | `execution-actor`, `execution_actor`                                                                                      | `stencila:executionActor`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                              | The id of the actor that the node was last executed by.              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionEnded`        | `execution-ended`, `execution_ended`                                                                                      | `stencila:executionEnded`                    | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                        | The timestamp when the last execution ended.                         | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionDuration`     | `execution-duration`, `execution_duration`                                                                                | `stencila:executionDuration`                 | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                          | Duration of the last execution.                                      | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `executionErrors`       | `execution-errors`, `execution_errors`, `executionError`, `execution-error`, `execution_error`                            | `stencila:executionErrors`                   | [`ExecutionError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/execution-error.md)*            | Errors when executing the node.                                      | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)    |
| `messages`              | `message`                                                                                                                 | `stencila:messages`                          | [`Message`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/message.md)*                          | Messages involved in the instruction.                                | [`Instruction`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction.md) |
| `assignee`              | -                                                                                                                         | `stencila:assignee`                          | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                              | An identifier for the agent assigned to perform the instruction      | [`Instruction`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction.md) |
| `authors`               | `author`                                                                                                                  | [`schema:author`](https://schema.org/author) | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                            | The authors of the instruction.                                      | [`Instruction`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction.md) |
| `content`               | -                                                                                                                         | `stencila:content`                           | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                              | The content to which the instruction applies.                        | -                                                                                                          |
| `suggestion`            | -                                                                                                                         | `stencila:suggestion`                        | [`SuggestionBlockType`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-block-type.md) | A suggestion for the instruction                                     | -                                                                                                          |

## Related

The `InstructionBlock` type is related to these types:

- Parents: [`Instruction`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction.md)
- Children: none

## Formats

The `InstructionBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes                              |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.md)           | 🟢 No loss    |           | 🚧 Under development |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |              |           | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            | Encoded using implemented function |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |                                    |

## Bindings

The `InstructionBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.org/InstructionBlock.jsonld)
- [JSON Schema](https://stencila.org/InstructionBlock.schema.json)
- Python class [`InstructionBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/instruction_block.py)
- Rust struct [`InstructionBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruction_block.rs)
- TypeScript class [`InstructionBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/InstructionBlock.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `InstructionBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property   | Complexity | Description                                               | Strategy                                  |
| ---------- | ---------- | --------------------------------------------------------- | ----------------------------------------- |
| `assignee` | Min+       | No assignee                                               | `None`                                    |
|            | Low+       | Generate an arbitrary id using expected characters        | `option::of(r"[a-zA-Z][a-zA-Z\-_/.@]")`   |
|            | Max        | Generate an arbitrary assignee id using any characters    | `option::of(String::arbitrary())`         |
| `content`  | Min+       | No content                                                | `None`                                    |
|            | Low+       | Generate a single arbitrary, non-recursive, block node    | `option::of(vec_blocks_non_recursive(1))` |
|            | High+      | Generate up to two arbitrary, non-recursive, block nodes  | `option::of(vec_blocks_non_recursive(2))` |
|            | Max        | Generate up to four arbitrary, non-recursive, block nodes | `option::of(vec_blocks_non_recursive(4))` |

## Source

This documentation was generated from [`InstructionBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/InstructionBlock.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.