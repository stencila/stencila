---
title: Research Object Relation Kind
description: The kind of relation from one research object to another.
---

This enumeration defines the constrained vocabulary for authored relations
between Stencila research objects. It is based on
[MIRA](https://github.com/MIRA-science/schema), a discourse graph vocabulary
for modular research objects with JSON-LD context published at
`https://purl.org/mira-science/mira.jsonld`.

Variant `@id`s mirror the published MIRA term identifiers verbatim, which is
why some are camelCase (`mira:supportedBy`) and others snake_case
(`mira:is_grounded_in`).

Most relations come in directional pairs declared as `owl:inverseOf` each
other in MIRA (e.g. `Supports`/`SupportedBy`). Both directions are
authorable so that a relation can be declared on whichever research object
is most convenient to edit; the direction is always semantically
significant. `Follows` has no inverse in MIRA. `RequestFor` and
`RequestTarget` are not inverses of each other: both have a request as
their source and a request may declare both.

Use this narrower vocabulary on `ResearchObjectRelation`. Values map
one-to-one by name to the corresponding discourse-oriented variants in
`GraphEdgeKind` when document graphs are built. Variant descriptions are
duplicated in `GraphEdgeKind` and must be kept in sync.


# Members

The `ResearchObjectRelationKind` type has these members:

| Member          | Description                                                                                                                                |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `Supports`      | The source research object, typically evidence or a claim, supports the target claim.                                                      |
| `SupportedBy`   | The source claim is supported by the target research object, typically evidence or another claim. Inverse of `Supports`.                   |
| `Opposes`       | The source research object, typically evidence or a claim, opposes the target claim.                                                       |
| `OpposedBy`     | The source claim is opposed by the target research object, typically evidence or another claim. Inverse of `Opposes`.                      |
| `Addresses`     | The source claim addresses the target research question.                                                                                   |
| `AddressedBy`   | The source research question is addressed by the target claim. Inverse of `Addresses`.                                                     |
| `Follows`       | The source research object, typically a study or other research activity, was conducted following the target protocol.                     |
| `Grounds`       | The source research object, typically a study or other research activity, produced and grounds the target evidence.                        |
| `IsGroundedIn`  | The source evidence is grounded in the target study or other research activity that produced it. Inverse of `Grounds`.                     |
| `RequestFor`    | The source request asks for the target work, such as a study or protocol execution, to be carried out. Not the inverse of `RequestTarget`. |
| `RequestTarget` | The source request concerns the target claim that the requested work may elucidate. Not the inverse of `RequestFor`.                       |

# Bindings

The `ResearchObjectRelationKind` type is represented in:

- [JSON-LD](https://stencila.org/ResearchObjectRelationKind.jsonld)
- [JSON Schema](https://stencila.org/ResearchObjectRelationKind.schema.json)
- Python type [`ResearchObjectRelationKind`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ResearchObjectRelationKind`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/research_object_relation_kind.rs)
- TypeScript type [`ResearchObjectRelationKind`](https://github.com/stencila/stencila/blob/main/ts/src/types/ResearchObjectRelationKind.ts)

***

This documentation was generated from [`ResearchObjectRelationKind.yaml`](https://github.com/stencila/stencila/blob/main/schema/ResearchObjectRelationKind.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
