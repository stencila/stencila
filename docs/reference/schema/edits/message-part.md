---
title: Message Part
description: A union type for a part of a message.
config:
  publish:
    ghost:
      type: post
      slug: message-part
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Edits
---

This union type uses `Text`, instead of `string`, so that each type of part
is an entity with a type and node id.


# Members

The `MessagePart` type has these members:

- [`Text`](https://stencila.ghost.io/docs/reference/schema/text)
- [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)
- [`AudioObject`](https://stencila.ghost.io/docs/reference/schema/audio-object)
- [`VideoObject`](https://stencila.ghost.io/docs/reference/schema/video-object)
- [`File`](https://stencila.ghost.io/docs/reference/schema/file)

# Bindings

The `MessagePart` type is represented in:

- [JSON-LD](https://stencila.org/MessagePart.jsonld)
- [JSON Schema](https://stencila.org/MessagePart.schema.json)
- Python type [`MessagePart`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/message_part.py)
- Rust type [`MessagePart`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/message_part.rs)
- TypeScript type [`MessagePart`](https://github.com/stencila/stencila/blob/main/ts/src/types/MessagePart.ts)

# Source

This documentation was generated from [`MessagePart.yaml`](https://github.com/stencila/stencila/blob/main/schema/MessagePart.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
