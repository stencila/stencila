---
title: Research Object Relation
description: A relation from one research object to another.
---

This type is used on `ResearchObject.relations` for authored, source-local
relations between concrete research objects in a document.

The source is the research object that owns the relation. The target is a
string reference to another research object or external resource, such as
`#claim-1`, `evidence-1`, a graph node id, or an absolute URI.

Targets are resolved during graph construction as follows: a leading `#` is
optional and stripped, so `#claim-1` and `claim-1` are equivalent; the
reference is resolved first against declared research object ids, then
against existing graph node ids; otherwise, a value with a URI scheme (e.g.
`https://...`) is resolved to an external resource node. Unresolvable
targets do not produce graph edges.


# Properties

The `ResearchObjectRelation` type has these properties:

| Name     | Description                                      | Type                                                               | Inherited from          |
| -------- | ------------------------------------------------ | ------------------------------------------------------------------ | ----------------------- |
| `kind`   | The kind of relation.                            | [`ResearchObjectRelationKind`](./research-object-relation-kind.md) | -                       |
| `target` | The target research object or external resource. | [`String`](./string.md)                                            | -                       |
| `id`     | The identifier for this item.                    | [`String`](./string.md)                                            | [`Entity`](./entity.md) |

# Related

The `ResearchObjectRelation` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ResearchObjectRelation` type is represented in:

- [JSON-LD](https://stencila.org/ResearchObjectRelation.jsonld)
- [JSON Schema](https://stencila.org/ResearchObjectRelation.schema.json)
- Python class [`ResearchObjectRelation`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ResearchObjectRelation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/research_object_relation.rs)
- TypeScript class [`ResearchObjectRelation`](https://github.com/stencila/stencila/blob/main/ts/src/types/ResearchObjectRelation.ts)

***

This documentation was generated from [`ResearchObjectRelation.yaml`](https://github.com/stencila/stencila/blob/main/schema/ResearchObjectRelation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
