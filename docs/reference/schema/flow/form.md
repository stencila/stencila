# Form

**A form to batch updates in document parameters**

**`@id`**: `stencila:Form`

This type is marked as unstable and is subject to change.

## Properties

The `Form` type has these properties:

| Name                  | `@id`                                | Type                                                                                                                                                                                                | Description                                                                               | Inherited from                                                                                          |
| --------------------- | ------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| id                    | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                     | The identifier for this item                                                              | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)        |
| executionAuto         | `stencila:executionAuto`             | [`ExecutionAuto`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-auto.md)                                                                                      | Under which circumstances the code should be automatically executed.                      | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| compilationDigest     | `stencila:compilationDigest`         | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md)                                                                                  | A digest of the content, semantics and dependencies of the node.                          | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDigest       | `stencila:executionDigest`           | [`ExecutionDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-digest.md)                                                                                  | The `compileDigest` of the node when it was last executed.                                | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDependencies | `stencila:executionDependencies`     | [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependency.md)*                                                                         | The upstream dependencies of this node.                                                   | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDependants   | `stencila:executionDependants`       | [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md)*                                                                           | The downstream dependants of this node.                                                   | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionTags         | `stencila:executionTags`             | [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-tag.md)*                                                                                       | Tags in the code which affect its execution                                               | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionCount        | `stencila:executionCount`            | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                                                                                                   | A count of the number of times that the node has been executed.                           | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionRequired     | `stencila:executionRequired`         | [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-required.md)                                                                              | Whether, and why, the code requires execution or re-execution.                            | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionKernel       | `stencila:executionKernel`           | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                     | The id of the kernel that the node was last executed in.                                  | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionStatus       | `stencila:executionStatus`           | [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-status.md)                                                                                  | Status of the most recent, including any current, execution.                              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionEnded        | `stencila:executionEnded`            | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                                                                                               | The timestamp when the last execution ended.                                              | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| executionDuration     | `stencila:executionDuration`         | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                                                                                                 | Duration of the last execution.                                                           | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| errors                | `stencila:errors`                    | [`CodeError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-error.md)*                                                                                             | Errors when compiling (e.g. syntax errors) or executing the node.                         | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| content               | `stencila:content`                   | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                                                                                                     | The content within the form, usually containing at least one `Parameter`.                 | [`Form`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form.md)             |
| deriveFrom            | `stencila:deriveFrom`                | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                     | The dotted path to the object (e.g a database table) that the form should be derived from | [`Form`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form.md)             |
| deriveAction          | `stencila:deriveAction`              | [`FormDeriveAction`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form-derive-action.md)                                                                               | The action (create, update or delete) to derive for the form                              | [`Form`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form.md)             |
| deriveItem            | `stencila:deriveItem`                | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md) \| [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | An identifier for the item to be the target of Update or Delete actions                   | [`Form`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form.md)             |

## Related

The `Form` type is related to these types:

- Parents: [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)
- Children: none

## Formats

The `Form` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `Form` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Form.jsonld)
- [JSON Schema](https://stencila.dev/Form.schema.json)
- Python class [`Form`](https://github.com/stencila/stencila/blob/main/python/stencila/types/form.py)
- Rust struct [`Form`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/form.rs)
- TypeScript class [`Form`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Form.ts)

## Source

This documentation was generated from [`Form.yaml`](https://github.com/stencila/stencila/blob/main/schema/Form.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).