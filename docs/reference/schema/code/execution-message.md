# Execution Message

**An error, warning or log message generated during execution.**

**`@id`**: `stencila:ExecutionMessage`

This type is marked as unstable and is subject to change.

## Properties

The `ExecutionMessage` type has these properties:

| Name           | Aliases                               | `@id`                                | Type                                                                                                          | Description                                                          | Inherited from                                                                                   |
| -------------- | ------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`           | -                                     | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)               | The identifier for this item.                                        | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `level`        | -                                     | `stencila:level`                     | [`MessageLevel`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/message-level.md) | The severity level of the message.                                   | -                                                                                                |
| `message`      | -                                     | `stencila:message`                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)               | The text of the message.                                             | -                                                                                                |
| `errorType`    | `error-type`, `error_type`            | `stencila:errorType`                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)               | The type of error e.g. "SyntaxError", "ZeroDivisionError".           | -                                                                                                |
| `codeLocation` | `code-location`, `code_location`      | `stencila:codeLocation`              | [`CodeLocation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/code-location.md)  | The location that the error occurred or other message emanated from. | -                                                                                                |
| `stackTrace`   | `trace`, `stack-trace`, `stack_trace` | `stencila:stackTrace`                | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)               | Stack trace leading up to the error.                                 | -                                                                                                |

## Related

The `ExecutionMessage` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `ExecutionMessage` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | ⚠️ High loss |            | 🔶 Beta              |       |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |       |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |       |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |       |
| [IPYNB](https://github.com/stencila/stencila/blob/main/docs/reference/formats/ipynb.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |       |

## Bindings

The `ExecutionMessage` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ExecutionMessage.jsonld)
- [JSON Schema](https://stencila.org/ExecutionMessage.schema.json)
- Python class [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_message.py)
- Rust struct [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_message.rs)
- TypeScript class [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionMessage.ts)

## Source

This documentation was generated from [`ExecutionMessage.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionMessage.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
