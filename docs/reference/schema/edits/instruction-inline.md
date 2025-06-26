---
title: Instruction Inline
description: An instruction to edit some inline content.
config:
  publish:
    ghost:
      type: post
      slug: instruction-inline
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Edits
---

# Properties

The `InstructionInline` type has these properties:

| Name                    | Description                                                                | Type                                                                                           | Inherited from                                                               | `JSON-LD @id`                        | Aliases                                                                                                                   |
| ----------------------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)           | [`schema:id`](https://schema.org/id) | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.                     | [`ExecutionMode`](https://stencila.ghost.io/docs/reference/schema/execution-mode)              | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionMode`             | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.           | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:compilationDigest`         | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                               | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:compilationMessages`       | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.             | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionDigest`           | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                                    | [`ExecutionDependency`](https://stencila.ghost.io/docs/reference/schema/execution-dependency)* | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionDependencies`     | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                                    | [`ExecutionDependant`](https://stencila.ghost.io/docs/reference/schema/execution-dependant)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionDependants`       | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                               | [`ExecutionTag`](https://stencila.ghost.io/docs/reference/schema/execution-tag)*               | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionTags`             | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.            | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                           | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionCount`            | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.             | [`ExecutionRequired`](https://stencila.ghost.io/docs/reference/schema/execution-required)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionRequired`         | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.               | [`ExecutionStatus`](https://stencila.ghost.io/docs/reference/schema/execution-status)          | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionStatus`           | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution.           | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionInstance`         | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                               | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionEnded`            | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                            | [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)                         | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionDuration`         | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                                 | [`ExecutionMessage`](https://stencila.ghost.io/docs/reference/schema/execution-message)*       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)   | `stencila:executionMessages`         | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `instructionType`       | The type of instruction describing the operation to be performed.          | [`InstructionType`](https://stencila.ghost.io/docs/reference/schema/instruction-type)          | [`Instruction`](https://stencila.ghost.io/docs/reference/schema/instruction) | `stencila:instructionType`           | `instruction-type`, `instruction_type`                                                                                    |
| `prompt`                | The prompt selected, rendered and provided to the model                    | [`PromptBlock`](https://stencila.ghost.io/docs/reference/schema/prompt-block)                  | [`Instruction`](https://stencila.ghost.io/docs/reference/schema/instruction) | `stencila:prompt`                    | -                                                                                                                         |
| `message`               | The instruction message, possibly including images, audio, or other media. | [`InstructionMessage`](https://stencila.ghost.io/docs/reference/schema/instruction-message)    | [`Instruction`](https://stencila.ghost.io/docs/reference/schema/instruction) | `stencila:message`                   | -                                                                                                                         |
| `modelParameters`       | Model selection and inference parameters.                                  | [`ModelParameters`](https://stencila.ghost.io/docs/reference/schema/model-parameters)          | [`Instruction`](https://stencila.ghost.io/docs/reference/schema/instruction) | `stencila:modelParameters`           | `model-parameters`, `model_parameters`, `model-params`, `model_params`, `model-pars`, `model_pars`, `model`               |
| `activeSuggestion`      | The index of the suggestion that is currently active                       | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer)          | [`Instruction`](https://stencila.ghost.io/docs/reference/schema/instruction) | `stencila:activeSuggestion`          | `active-suggestion`, `active_suggestion`                                                                                  |
| `content`               | The content to which the instruction applies.                              | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)*                            | -                                                                            | `stencila:content`                   | -                                                                                                                         |
| `suggestions`           | Suggestions for the instruction                                            | [`SuggestionInline`](https://stencila.ghost.io/docs/reference/schema/suggestion-inline)*       | -                                                                            | `stencila:suggestions`               | `suggestion`                                                                                                              |

# Related

The `InstructionInline` type is related to these types:

- Parents: [`Instruction`](https://stencila.ghost.io/docs/reference/schema/instruction)
- Children: none

# Formats

The `InstructionInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding   | Support | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |            |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |            |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |            |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |            |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |            |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss |         |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |            |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss |         |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |            |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |            |         |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |            |         |

# Bindings

The `InstructionInline` type is represented in:

- [JSON-LD](https://stencila.org/InstructionInline.jsonld)
- [JSON Schema](https://stencila.org/InstructionInline.schema.json)
- Python class [`InstructionInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/instruction_inline.py)
- Rust struct [`InstructionInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruction_inline.rs)
- TypeScript class [`InstructionInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/InstructionInline.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `InstructionInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                | Strategy                                   |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------------------ |
| `content` | Min+       | No content                                                 | `None`                                     |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `option::of(vec_inlines_non_recursive(1))` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `option::of(vec_inlines_non_recursive(2))` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `option::of(vec_inlines_non_recursive(4))` |

# Source

This documentation was generated from [`InstructionInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/InstructionInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
