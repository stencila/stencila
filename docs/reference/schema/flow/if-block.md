# If Block

**Show and execute alternative content conditional upon an executed expression.**

**`@id`**: `stencila:IfBlock`

This type is marked as unstable and is subject to change.

## Properties

The `IfBlock` type has these properties:

| Name                    | Aliases                                                                                                                   | `@id`                                | Type                                                                                                                        | Description                                                      | Inherited from                                                                                          |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `id`                    | -                                                                                                                         | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The identifier for this item.                                    | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)        |
| `executionMode`         | `execution-mode`, `execution_mode`                                                                                        | `stencila:executionMode`             | [`ExecutionMode`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-mode.md)              | Under which circumstances the node should be executed.           | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionBounds`       | `execution-bounds`, `execution_bounds`                                                                                    | `stencila:executionBounds`           | [`ExecutionBounds`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-bounds.md)          | Under which circumstances child nodes should be executed.        | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `compilationDigest`     | `compilation-digest`, `compilation_digest`                                                                                | `stencila:compilationDigest`         | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)      | A digest of the content, semantics and dependencies of the node. | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `compilationMessages`   | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        | `stencila:compilationMessages`       | [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-message.md)*   | Messages generated while compiling the code.                     | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionDigest`       | `execution-digest`, `execution_digest`                                                                                    | `stencila:executionDigest`           | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)      | The `compilationDigest` of the node when it was last executed.   | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionDependencies` | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` | `stencila:executionDependencies`     | [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependency.md)* | The upstream dependencies of this node.                          | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionDependants`   | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        | `stencila:executionDependants`       | [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md)*   | The downstream dependants of this node.                          | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionTags`         | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      | `stencila:executionTags`             | [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-tag.md)*               | Tags in the code which affect its execution.                     | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionCount`        | `execution-count`, `execution_count`                                                                                      | `stencila:executionCount`            | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                           | A count of the number of times that the node has been executed.  | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionRequired`     | `execution-required`, `execution_required`                                                                                | `stencila:executionRequired`         | [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-required.md)      | Whether, and why, the code requires execution or re-execution.   | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionStatus`       | `execution-status`, `execution_status`                                                                                    | `stencila:executionStatus`           | [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-status.md)          | Status of the most recent, including any current, execution.     | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionInstance`     | `execution-instance`, `execution_instance`                                                                                | `stencila:executionInstance`         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | The id of the kernel instance that performed the last execution. | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionBounded`      | `execution-bounded`, `execution_bounded`                                                                                  | `stencila:executionBounded`          | [`ExecutionBounds`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-bounds.md)          | The bounds, if any, on the last execution.                       | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionEnded`        | `execution-ended`, `execution_ended`                                                                                      | `stencila:executionEnded`            | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                       | The timestamp when the last execution ended.                     | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionDuration`     | `execution-duration`, `execution_duration`                                                                                | `stencila:executionDuration`         | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                         | Duration of the last execution.                                  | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `executionMessages`     | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  | `stencila:executionMessages`         | [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/execution-message.md)*       | Messages emitted while executing the node.                       | [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md) |
| `clauses`               | `clause`                                                                                                                  | `stencila:clauses`                   | [`IfBlockClause`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if-block-clause.md)*            | The clauses making up the `IfBlock` node                         | -                                                                                                       |

## Related

The `IfBlock` type is related to these types:

- Parents: [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md)
- Children: none

## Formats

The `IfBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | ⚠️ High loss |            | 🔶 Beta              | Encoded using implemented function |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [IPYNB](https://github.com/stencila/stencila/blob/main/docs/reference/formats/ipynb.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [Lexical JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/lexical.md)     | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |                                    |
| [Koenig JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/koenig.md)       | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |                                    |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |                                    |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |                                    |

## Bindings

The `IfBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.org/IfBlock.jsonld)
- [JSON Schema](https://stencila.org/IfBlock.schema.json)
- Python class [`IfBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/if_block.py)
- Rust struct [`IfBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/if_block.rs)
- TypeScript class [`IfBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/IfBlock.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `IfBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                    | Strategy                                                   |
| --------- | ---------- | ---------------------------------------------- | ---------------------------------------------------------- |
| `clauses` | Min+       | A single fixed clause with a single paragraph. | `vec![ibc("true", None::<String>, [p([t("If clause")])])]` |
|           | Low+       | Generate up to 3 arbitrary if clauses          | `vec(IfBlockClause::arbitrary(), size_range(1..=3))`       |
|           | High+      | Generate up to 5 arbitrary if clauses          | `vec(IfBlockClause::arbitrary(), size_range(1..=10))`      |

## Source

This documentation was generated from [`IfBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/IfBlock.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
