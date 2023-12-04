# If Block Clause

**A clause within an `IfBlock` node.**

**`@id`**: `stencila:IfBlockClause`

This type is marked as unstable and is subject to change.

## Properties

The `IfBlockClause` type has these properties:

| Name                    | Aliases                                                                                                                   | `@id`                                                                  | Type                                                                                                                        | Description                                                           | Inherited from                                                                                                   |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `id`                    | -                                                                                                                         | [`schema:id`](https://schema.org/id)                                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The identifier for this item.                                         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)                 |
| `autoExec`              | `auto`, `auto-exec`, `auto_exec`                                                                                          | `stencila:autoExec`                                                    | [`AutomaticExecution`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/automatic-execution.md)    | Under which circumstances the code should be automatically executed.  | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `compilationDigest`     | `compilation-digest`, `compilation_digest`                                                                                | `stencila:compilationDigest`                                           | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)      | A digest of the content, semantics and dependencies of the node.      | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `compilationErrors`     | `compilation-errors`, `compilation_errors`, `compilationError`, `compilation-error`, `compilation_error`                  | `stencila:compilationErrors`                                           | [`CompilationError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-error.md)*       | Errors generated when compiling the code.                             | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionDigest`       | `execution-digest`, `execution_digest`                                                                                    | `stencila:executionDigest`                                             | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)      | The `compilationDigest` of the node when it was last executed.        | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionDependencies` | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` | `stencila:executionDependencies`                                       | [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependency.md)* | The upstream dependencies of this node.                               | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionDependants`   | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        | `stencila:executionDependants`                                         | [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md)*   | The downstream dependants of this node.                               | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionTags`         | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      | `stencila:executionTags`                                               | [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-tag.md)*               | Tags in the code which affect its execution.                          | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionCount`        | `execution-count`, `execution_count`                                                                                      | `stencila:executionCount`                                              | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                           | A count of the number of times that the node has been executed.       | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionRequired`     | `execution-required`, `execution_required`                                                                                | `stencila:executionRequired`                                           | [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-required.md)      | Whether, and why, the code requires execution or re-execution.        | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionKernel`       | `execution-kernel`, `execution_kernel`                                                                                    | `stencila:executionKernel`                                             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The id of the kernel that the node was last executed in.              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionStatus`       | `execution-status`, `execution_status`                                                                                    | `stencila:executionStatus`                                             | [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-status.md)          | Status of the most recent, including any current, execution.          | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionEnded`        | `execution-ended`, `execution_ended`                                                                                      | `stencila:executionEnded`                                              | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                       | The timestamp when the last execution ended.                          | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionDuration`     | `execution-duration`, `execution_duration`                                                                                | `stencila:executionDuration`                                           | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                         | Duration of the last execution.                                       | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `executionErrors`       | `execution-errors`, `execution_errors`, `executionError`, `execution-error`, `execution_error`                            | `stencila:executionErrors`                                             | [`ExecutionError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/execution-error.md)*           | Errors when executing the node.                                       | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)          |
| `code`                  | -                                                                                                                         | `stencila:code`                                                        | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                                 | The code.                                                             | [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-executable.md) |
| `programmingLanguage`   | `programming-language`, `programming_language`                                                                            | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The programming language of the code.                                 | [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-executable.md) |
| `isActive`              | `is-active`, `is_active`                                                                                                  | `stencila:isActive`                                                    | [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)                           | Whether this clause is the active clause in the parent `IfBlock` node | -                                                                                                                |
| `content`               | -                                                                                                                         | `stencila:content`                                                     | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                             | The content to render if the result is truthy                         | -                                                                                                                |

## Related

The `IfBlockClause` type is related to these types:

- Parents: [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-executable.md)
- Children: none

## Formats

The `IfBlockClause` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |                  |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss    | 🔶 Beta                 |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |       |

## Bindings

The `IfBlockClause` type is represented in these bindings:

- [JSON-LD](https://stencila.org/IfBlockClause.jsonld)
- [JSON Schema](https://stencila.org/IfBlockClause.schema.json)
- Python class [`IfBlockClause`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/if_block_clause.py)
- Rust struct [`IfBlockClause`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/if_block_clause.rs)
- TypeScript class [`IfBlockClause`](https://github.com/stencila/stencila/blob/main/ts/src/types/IfBlockClause.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `IfBlockClause` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property              | Complexity | Description                                                                                                                     | Strategy                                   |
| --------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| `code`                | Min+       | Generate a simple fixed string of code.                                                                                         | `Cord::new("code")`                        |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (excludes whitespace which<br><br>can be problematic in Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::new)` |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                | `r"[^\p{C}]{1,100}".prop_map(Cord::new)`   |
|                       | Max        | Generate an arbitrary string.                                                                                                   | `String::arbitrary().prop_map(Cord::new)`  |
| `programmingLanguage` | Min+       | Generate a simple fixed string.                                                                                                 | `Some(String::from("lang"))`               |
|                       | Low+       | Generate one of the well known programming language short names.                                                                | `option::of(r"(cpp)\|(js)\|(py)\|(r)\|(ts)")` |
|                       | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                   | `option::of(r"[a-zA-Z0-9]{1,10}")`         |
|                       | Max        | Generate an arbitrary string.                                                                                                   | `option::of(String::arbitrary())`          |
| `content`             | Min+       | A single, simple paragraph.                                                                                                     | `vec![p([t("If clause content")])]`        |
|                       | Low+       | Generate up to two arbitrary, non-recursive, block nodes.                                                                       | `vec_blocks_non_recursive(2)`              |
|                       | Max        | Generate up to four arbitrary, non-recursive, block nodes.                                                                      | `vec_blocks_non_recursive(4)`              |

## Source

This documentation was generated from [`IfBlockClause.yaml`](https://github.com/stencila/stencila/blob/main/schema/IfBlockClause.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.