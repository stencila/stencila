---
title: Prompt Block
description: A preview of how a prompt will be rendered at a location in the document
---

Used on an `Instruction` to render a prompt and display the rendering to the user.
Can also be used standalone to preview how a prompt is rendered at a particular
position in a document.


This type is marked as unstable and is subject to change.

# Properties

The `PromptBlock` type has these properties:

| Name                    | Description                                                      | Type                                                | Inherited from                  | `JSON-LD @id`                                | Aliases                                                                                                                   |
| ----------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                    | [`String`](./string.md)                             | [`Entity`](./entity.md)         | [`schema:id`](https://schema.org/id)         | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.           | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md) | `stencila:executionMode`                     | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node. | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                     | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md) | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.   | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) | `stencila:executionDigest`                   | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                          | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md) | `stencila:executionDependencies`             | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                          | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md) | `stencila:executionDependants`               | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                     | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md) | `stencila:executionTags`                     | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.  | [`Integer`](./integer.md)                           | [`Executable`](./executable.md) | `stencila:executionCount`                    | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.   | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md) | `stencila:executionRequired`                 | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.     | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md) | `stencila:executionStatus`                   | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution. | [`String`](./string.md)                             | [`Executable`](./executable.md) | `stencila:executionInstance`                 | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                     | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md) | `stencila:executionEnded`                    | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                  | [`Duration`](./duration.md)                         | [`Executable`](./executable.md) | `stencila:executionDuration`                 | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                       | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md) | `stencila:executionMessages`                 | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `instructionType`       | The type of instruction the  being used for                      | [`InstructionType`](./instruction-type.md)          | -                               | `stencila:instructionType`                   | `instruction-type`, `instruction_type`                                                                                    |
| `nodeTypes`             | The type of nodes the prompt is being used for                   | [`String`](./string.md)*                            | -                               | `stencila:nodeTypes`                         | `node-types`, `node_types`, `nodeType`, `node-type`, `node_type`                                                          |
| `relativePosition`      | The relative position of the node being edited, described etc.   | [`RelativePosition`](./relative-position.md)        | -                               | `stencila:relativePosition`                  | `relative-position`, `relative_position`                                                                                  |
| `query`                 | A user text query used to infer the `target` prompt              | [`String`](./string.md)                             | -                               | `stencila:query`                             | -                                                                                                                         |
| `target`                | An identifier for the prompt to be rendered                      | [`String`](./string.md)                             | -                               | [`schema:target`](https://schema.org/target) | -                                                                                                                         |
| `directory`             | The home directory of the prompt                                 | [`String`](./string.md)                             | -                               | `stencila:directory`                         | -                                                                                                                         |
| `content`               | The executed content of the prompt                               | [`Block`](./block.md)*                              | -                               | `stencila:content`                           | -                                                                                                                         |

# Related

The `PromptBlock` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: none

# Formats

The `PromptBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                    |
| [JATS](../formats/jats.md)                       |              |              |                                    |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                    |
| [CSV](../formats/csv.md)                         |              |              |                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                    |
| [Directory](../formats/directory.md)             |              |              |                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                    |

# Bindings

The `PromptBlock` type is represented in:

- [JSON-LD](https://stencila.org/PromptBlock.jsonld)
- [JSON Schema](https://stencila.org/PromptBlock.schema.json)
- Python class [`PromptBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/prompt_block.py)
- Rust struct [`PromptBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/prompt_block.rs)
- TypeScript class [`PromptBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/PromptBlock.ts)

# Source

This documentation was generated from [`PromptBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/PromptBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
