---
title: Message Part
description: A union type for a part of a message.
---

This union type uses `Text`, instead of `string`, so that each type of part
is an entity with a type and node id.


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
- Python type [`MessagePart`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/message_part.py)
- Rust type [`MessagePart`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/message_part.rs)
- TypeScript type [`MessagePart`](https://github.com/stencila/stencila/blob/main/ts/src/types/MessagePart.ts)

# Source

This documentation was generated from [`MessagePart.yaml`](https://github.com/stencila/stencila/blob/main/schema/MessagePart.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
