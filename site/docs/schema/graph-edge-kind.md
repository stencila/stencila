---
title: Graph Edge Kind
description: The kind of directed relationship represented by a graph edge.
---

This enumeration is used by `GraphEdge` for describing the relation between
nodes in a `Graph`. Edge direction is always from `GraphEdge.source` to
`GraphEdge.target`. For resource-flow relationships this is usually from
upstream dependency, source, or component to downstream dependant, result, or
containing node. For discourse relationships this is the authored relation
direction.

Resource-flow variant names should be read in graph direction as "upstream
<kind> downstream". Some names use passive phrasing (`UsedBy`, `ReadBy`) and
others use established relation phrases (`PartOf`, `Generated`, `Declares`),
but their direction is always defined by `GraphEdge.source` to
`GraphEdge.target`. Discourse variant names should be read as "source <kind>
target".

Variant descriptions document related W3C PROV-O predicates where applicable.
These mappings are guidance only: Stencila graph edges keep a consistent
dependency direction, while several PROV-O predicates use the opposite RDF
subject/object direction or apply only approximately. The PROV-O namespace is
`http://www.w3.org/ns/prov#`.

The discourse variants (`Supports` through `RequestTarget`) mirror
`ResearchObjectRelationKind`, which is the source of truth for their
semantics and MIRA-based `@id`s; the two enumerations must be kept in sync.


# Members

The `GraphEdgeKind` type has these members:

| Member          | Description                                                                                                                                                                                             |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `UsedBy`        | The upstream node is used by the downstream node. Related to `prov:used` with inverse direction.                                                                                                        |
| `ReadBy`        | The upstream node is read by the downstream node. Related to `prov:used` with inverse direction.                                                                                                        |
| `Generated`     | The upstream node generated the downstream node. Related to `prov:generated` and to `prov:wasGeneratedBy` with inverse direction.                                                                       |
| `WrittenTo`     | The upstream data or value is written to the downstream resource. Related to `prov:generated` for the downstream resource and to `prov:used` for the upstream value.                                    |
| `DerivedInto`   | The upstream node is derived into the downstream node. Related to `prov:wasDerivedFrom` with inverse direction.                                                                                         |
| `ConvertedInto` | The upstream node is converted into the downstream node, usually changing representation or media format. Related to `prov:wasDerivedFrom` with inverse direction.                                      |
| `CalledBy`      | The upstream node is called by the downstream node. Approximately related to `prov:used` with inverse direction, or `prov:wasInformedBy` for activity dependencies.                                     |
| `ImportedBy`    | The upstream node is imported by the downstream node. Approximately related to `prov:hadPrimarySource` or `prov:wasDerivedFrom` with inverse direction.                                                 |
| `PartOf`        | The upstream node is part of the downstream node. Related to `schema:isPartOf` and inverse `schema:hasPart`.                                                                                            |
| `IncludedBy`    | The upstream source is included by the downstream document node or document region.                                                                                                                     |
| `LinkedBy`      | The upstream resource is linked to by the downstream link, media object, document, or document region.                                                                                                  |
| `CitedBy`       | The upstream node is cited by the downstream node. Stencila document relation with no exact core PROV-O equivalent; quotation-specific cases may relate to `prov:wasQuotedFrom` with inverse direction. |
| `Declares`      | The upstream manifest, configuration, or source file declares the downstream environment, dependency, workflow, or other computational resource.                                                        |
| `Configures`    | The upstream configuration file or setting configures the downstream workflow, environment, code unit, or tool.                                                                                         |
| `RequiredBy`    | The upstream package or software component is required by the downstream environment or source code.                                                                                                    |
| `Pins`          | The upstream lockfile, digest, or exact version pin constrains the downstream environment or dependency.                                                                                                |
| `Supports`      | The source research object, typically evidence or a claim, supports the target claim.                                                                                                                   |
| `SupportedBy`   | The source claim is supported by the target research object, typically evidence or another claim. Inverse of `Supports`.                                                                                |
| `Opposes`       | The source research object, typically evidence or a claim, opposes the target claim.                                                                                                                    |
| `OpposedBy`     | The source claim is opposed by the target research object, typically evidence or another claim. Inverse of `Opposes`.                                                                                   |
| `Addresses`     | The source claim addresses the target research question.                                                                                                                                                |
| `AddressedBy`   | The source research question is addressed by the target claim. Inverse of `Addresses`.                                                                                                                  |
| `Follows`       | The source research object, typically a study or other research activity, was conducted following the target protocol.                                                                                  |
| `Grounds`       | The source research object, typically a study or other research activity, produced and grounds the target evidence.                                                                                     |
| `IsGroundedIn`  | The source evidence is grounded in the target study or other research activity that produced it. Inverse of `Grounds`.                                                                                  |
| `RequestFor`    | The source request asks for the target work, such as a study or protocol execution, to be carried out. Not the inverse of `RequestTarget`.                                                              |
| `RequestTarget` | The source request concerns the target claim that the requested work may elucidate. Not the inverse of `RequestFor`.                                                                                    |

# Bindings

The `GraphEdgeKind` type is represented in:

- [JSON-LD](https://stencila.org/GraphEdgeKind.jsonld)
- [JSON Schema](https://stencila.org/GraphEdgeKind.schema.json)
- Python type [`GraphEdgeKind`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`GraphEdgeKind`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_edge_kind.rs)
- TypeScript type [`GraphEdgeKind`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphEdgeKind.ts)

***

This documentation was generated from [`GraphEdgeKind.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphEdgeKind.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
