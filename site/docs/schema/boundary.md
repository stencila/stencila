---
title: Boundary
description: A positional boundary marker within inline content.
---

This is a type used in Stencila Schema for representing the boundary of a selection
or range within content.

It exists to support editing, review, and provenance workflows that need to
refer precisely to positions in document content without flattening the
document structure into plain text offsets. This makes it useful for
instructions, suggestions, and other document transformations.

Key properties identify the boundary location and its relationship to
surrounding content.


# Analogues

The following external types, elements, or nodes are similar to a `Boundary`:

- [DOM Range boundary point](https://dom.spec.whatwg.org/#concept-range-bp): Close conceptual analogue for a position at a node-and-offset boundary, though Stencila uses a document-model node rather than an ephemeral API object.
- [ProseMirror position marker](https://prosemirror.net/docs/guide/): Approximate editor-model analogue for addressing positions in structured content.

# Properties

The `Boundary` type has these properties:

| Name | Description                   | Type                    | Inherited from          |
| ---- | ----------------------------- | ----------------------- | ----------------------- |
| `id` | The identifier for this item. | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `Boundary` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Boundary` type is represented in:

- [JSON-LD](https://stencila.org/Boundary.jsonld)
- [JSON Schema](https://stencila.org/Boundary.schema.json)
- Python class [`Boundary`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Boundary`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/boundary.rs)
- TypeScript class [`Boundary`](https://github.com/stencila/stencila/blob/main/ts/src/types/Boundary.ts)

***

This documentation was generated from [`Boundary.yaml`](https://github.com/stencila/stencila/blob/main/schema/Boundary.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
