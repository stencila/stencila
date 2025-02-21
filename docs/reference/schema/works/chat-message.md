---
title: Chat Message
description: A message within a `Chat`.
config:
  publish:
    ghost:
      type: page
      slug: chat-message
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Works
---

This type is marked as unstable and is subject to change.

## Properties

The `ChatMessage` type has these properties:

| Name                    | Description                                                                   | Type                                                                                           | Inherited from                                                             | `JSON-LD @id`                                | Aliases                                                                                                                   |
| ----------------------- | ----------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                                 | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)         | [`schema:id`](https://schema.org/id)         | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.                        | [`ExecutionMode`](https://stencila.ghost.io/docs/reference/schema/execution-mode)              | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionMode`                     | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.              | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                                  | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.                | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDigest`                   | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                                       | [`ExecutionDependency`](https://stencila.ghost.io/docs/reference/schema/execution-dependency)* | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDependencies`             | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                                       | [`ExecutionDependant`](https://stencila.ghost.io/docs/reference/schema/execution-dependant)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDependants`               | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                                  | [`ExecutionTag`](https://stencila.ghost.io/docs/reference/schema/execution-tag)*               | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionTags`                     | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.               | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                           | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionCount`                    | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.                | [`ExecutionRequired`](https://stencila.ghost.io/docs/reference/schema/execution-required)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionRequired`                 | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.                  | [`ExecutionStatus`](https://stencila.ghost.io/docs/reference/schema/execution-status)          | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionStatus`                   | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution.              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionInstance`                 | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                                  | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionEnded`                    | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                               | [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)                         | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDuration`                 | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                                    | [`ExecutionMessage`](https://stencila.ghost.io/docs/reference/schema/execution-message)*       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionMessages`                 | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `author`                | The author of the message                                                     | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)                             | -                                                                          | [`schema:author`](https://schema.org/author) | -                                                                                                                         |
| `role`                  | The role of the message in the conversation.                                  | [`MessageRole`](https://stencila.ghost.io/docs/reference/schema/message-role)                  | -                                                                          | `stencila:role`                              | -                                                                                                                         |
| `content`               | The content of the message.                                                   | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*                              | -                                                                          | `stencila:content`                           | -                                                                                                                         |
| `files`                 | The content of the message.                                                   | [`File`](https://stencila.ghost.io/docs/reference/schema/file)*                                | -                                                                          | `stencila:files`                             | `file`                                                                                                                    |
| `isSelected`            | Whether this message is the selected message in the parent `ChatMessageGroup` | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)                           | -                                                                          | `stencila:isSelected`                        | `is-selected`, `is_selected`                                                                                              |

## Related

The `ChatMessage` type is related to these types:

- Parents: [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)
- Children: none

## Formats

The `ChatMessage` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support                            | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                |              |            |                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | ⚠️ High loss |            | Encoded using implemented function |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | ⚠️ High loss |            |                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | ⚠️ High loss |            |                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | ⚠️ High loss |            |                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | ⚠️ High loss |            |                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |                                    |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |                                    |
| [CBOR+Zstandard](https://stencila.ghost.io/docs/reference/formats/cbor.zstd) | 🟢 No loss    | 🟢 No loss  |                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |                                    |

## Bindings

The `ChatMessage` type is represented in:

- [JSON-LD](https://stencila.org/ChatMessage.jsonld)
- [JSON Schema](https://stencila.org/ChatMessage.schema.json)
- Python class [`ChatMessage`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/chat_message.py)
- Rust struct [`ChatMessage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/chat_message.rs)
- TypeScript class [`ChatMessage`](https://github.com/stencila/stencila/blob/main/ts/src/types/ChatMessage.ts)

## Source

This documentation was generated from [`ChatMessage.yaml`](https://github.com/stencila/stencila/blob/main/schema/ChatMessage.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
