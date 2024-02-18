# Instruction Message

**A message within an `Instruction`.**

**`@id`**: `stencila:InstructionMessage`

## Properties

The `InstructionMessage` type has these properties:

| Name      | Aliases  | `@id`                                            | Type                                                                                                                                                                                                                                                                                                                                               | Description                        | Inherited from                                                                                   |
| --------- | -------- | ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`      | -        | [`schema:id`](https://schema.org/id)             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                                                                                                                                                    | The identifier for this item.      | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `parts`   | `part`   | [`schema:hasParts`](https://schema.org/hasParts) | [`MessagePart`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/message-part.md)*                                                                                                                                                                                                                                       | Parts of the message.              | -                                                                                                |
| `content` | -        | `stencila:content`                               | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                                                                                                                                                                                                                                                    | Content of the message.            | -                                                                                                |
| `authors` | `author` | [`schema:author`](https://schema.org/author)     | ([`Person`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md) \| [`Organization`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md) \| [`SoftwareApplication`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software-application.md))* | The authors of the message.        | -                                                                                                |
| `level`   | -        | `stencila:level`                                 | [`MessageLevel`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/message-level.md)                                                                                                                                                                                                                                      | The severity level of the message. | -                                                                                                |

## Related

The `InstructionMessage` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `InstructionMessage` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.md)           | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)    |              |           | 🚧 Under development |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `InstructionMessage` type is represented in these bindings:

- [JSON-LD](https://stencila.org/InstructionMessage.jsonld)
- [JSON Schema](https://stencila.org/InstructionMessage.schema.json)
- Python class [`InstructionMessage`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/instruction_message.py)
- Rust struct [`InstructionMessage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruction_message.rs)
- TypeScript class [`InstructionMessage`](https://github.com/stencila/stencila/blob/main/ts/src/types/InstructionMessage.ts)

## Source

This documentation was generated from [`InstructionMessage.yaml`](https://github.com/stencila/stencila/blob/main/schema/InstructionMessage.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).