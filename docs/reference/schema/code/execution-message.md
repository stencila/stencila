---
title: Execution Message
description: An error, warning or log message generated during execution.
config:
  publish:
    ghost:
      type: post
      slug: execution-message
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Code
---

# Properties

The `ExecutionMessage` type has these properties:

| Name           | Description                                                          | Type                                                                            | Inherited from                                                     | `JSON-LD @id`                        | Aliases                               |
| -------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ------------------------------------- |
| `id`           | The identifier for this item.                                        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)              | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                                     |
| `level`        | The severity level of the message.                                   | [`MessageLevel`](https://stencila.ghost.io/docs/reference/schema/message-level) | -                                                                  | `stencila:level`                     | -                                     |
| `message`      | The text of the message.                                             | [`String`](https://stencila.ghost.io/docs/reference/schema/string)              | -                                                                  | `stencila:message`                   | -                                     |
| `errorType`    | The type of error e.g. "SyntaxError", "ZeroDivisionError".           | [`String`](https://stencila.ghost.io/docs/reference/schema/string)              | -                                                                  | `stencila:errorType`                 | `error-type`, `error_type`            |
| `codeLocation` | The location that the error occurred or other message emanated from. | [`CodeLocation`](https://stencila.ghost.io/docs/reference/schema/code-location) | -                                                                  | `stencila:codeLocation`              | `code-location`, `code_location`      |
| `stackTrace`   | Stack trace leading up to the error.                                 | [`String`](https://stencila.ghost.io/docs/reference/schema/string)              | -                                                                  | `stencila:stackTrace`                | `trace`, `stack-trace`, `stack_trace` |

# Related

The `ExecutionMessage` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `ExecutionMessage` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                |              |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | ⚠️ High loss |            |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | ⚠️ High loss |            |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | ⚠️ High loss |            |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | ⚠️ High loss |            |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | ⚠️ High loss |            |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)      | 🟢 No loss    | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |         |

# Bindings

The `ExecutionMessage` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionMessage.jsonld)
- [JSON Schema](https://stencila.org/ExecutionMessage.schema.json)
- Python class [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_message.py)
- Rust struct [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_message.rs)
- TypeScript class [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionMessage.ts)

# Source

This documentation was generated from [`ExecutionMessage.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionMessage.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
