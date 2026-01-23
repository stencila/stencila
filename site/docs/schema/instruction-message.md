---
title: Instruction Message
description: A message within an `Instruction`.
---

# Properties

The `InstructionMessage` type has these properties:

| Name         | Description                                                                     | Type                                        | Inherited from          |
| ------------ | ------------------------------------------------------------------------------- | ------------------------------------------- | ----------------------- |
| `id`         | The identifier for this item.                                                   | [`String`](./string.md)                     | [`Entity`](./entity.md) |
| `role`       | The role of the message in the conversation.                                    | [`MessageRole`](./message-role.md)          | -                       |
| `content`    | The content of the message as inline nodes.                                     | [`Inline`](./inline.md)*                    | -                       |
| `files`      | Files attached to the message.                                                  | [`File`](./file.md)*                        | -                       |
| `authors`    | The authors of the message.                                                     | [`Author`](./author.md)*                    | -                       |
| `provenance` | A summary of the provenance of the messages and content within the instruction. | [`ProvenanceCount`](./provenance-count.md)* | -                       |

# Related

The `InstructionMessage` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `InstructionMessage` type is represented in:

- [JSON-LD](https://stencila.org/InstructionMessage.jsonld)
- [JSON Schema](https://stencila.org/InstructionMessage.schema.json)
- Python class [`InstructionMessage`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`InstructionMessage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruction_message.rs)
- TypeScript class [`InstructionMessage`](https://github.com/stencila/stencila/blob/main/ts/src/types/InstructionMessage.ts)

***

This documentation was generated from [`InstructionMessage.yaml`](https://github.com/stencila/stencila/blob/main/schema/InstructionMessage.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
