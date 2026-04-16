---
title: Message Part
description: A union type for message parts.
---

This is a union type used in Stencila Schema for structured message content.

It groups together the node types that can appear as parts of a message,
including text and media attachments. Using [`Text`](./text.md) rather than a
primitive string keeps each part as a node with its own type and identity.

Use this type to understand what content can appear in structured instruction
and chat messages.


# Analogues

The following external types, elements, or nodes are similar to a `MessagePart`:

- [multimodal message content part](https://platform.openai.com/docs/guides/images): Close analogue for typed message parts combining text and media attachments within one structured message.

# Members

The `MessagePart` type has these members:

- [`Text`](./text.md)
- [`ImageObject`](./image-object.md)
- [`AudioObject`](./audio-object.md)
- [`VideoObject`](./video-object.md)
- [`File`](./file.md)

# Bindings

The `MessagePart` type is represented in:

- [JSON-LD](https://stencila.org/MessagePart.jsonld)
- [JSON Schema](https://stencila.org/MessagePart.schema.json)
- Python type [`MessagePart`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`MessagePart`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/message_part.rs)
- TypeScript type [`MessagePart`](https://github.com/stencila/stencila/blob/main/ts/src/types/MessagePart.ts)

***

This documentation was generated from [`MessagePart.yaml`](https://github.com/stencila/stencila/blob/main/schema/MessagePart.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
