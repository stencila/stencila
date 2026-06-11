---
title: Execute Action
description: An action that executes code, a prompt, a workflow, or another executable node.
---

This is a Stencila extension of schema.org [`Action`](./action.md).

It represents computational execution as a first-class provenance activity.
In graphs, execution actions are usually attached to `Generated` edges from
executable nodes to their recorded outputs.


# Analogues

The following external types, elements, or nodes are similar to a `ExecuteAction`:

- [PROV Activity](https://www.w3.org/TR/prov-o/#Activity): Approximate provenance analogue for an execution activity.
- [C2PA org.stencila.executed action](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_actions): Stencila-specific C2PA action projection for executable document nodes.

# Properties

The `ExecuteAction` type has these properties:

| Name              | Description                                                                      | Type                                                                   | Inherited from          |
| ----------------- | -------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ----------------------- |
| `actionStatus`    | The current status of the action.                                                | [`ActionStatusType`](./action-status-type.md)                          | [`Action`](./action.md) |
| `agent`           | The direct performer or driver of the action.                                    | [`ActionAgent`](./action-agent.md)                                     | [`Action`](./action.md) |
| `participants`    | Other agents that participated in the action.                                    | [`ActionAgent`](./action-agent.md)*                                    | [`Action`](./action.md) |
| `provider`        | The service provider, service operator, or performer responsible for the action. | [`ActionAgent`](./action-agent.md)                                     | [`Action`](./action.md) |
| `objects`         | The objects or input values upon which the action is carried out.                | [`Node`](./node.md)*                                                   | [`Action`](./action.md) |
| `results`         | The objects or values produced by the action.                                    | [`Node`](./node.md)*                                                   | [`Action`](./action.md) |
| `instrument`      | The object, software, or other instrument that helped perform the action.        | [`ThingVariant`](./thing-variant.md) \| [`String`](./string.md)        | [`Action`](./action.md) |
| `environment`     | Environment variables or settings that affected the action.                      | [`PropertyValue`](./property-value.md)*                                | [`Action`](./action.md) |
| `containerImages` | Container images used by the action.                                             | ([`ContainerImage`](./container-image.md) \| [`String`](./string.md))* | [`Action`](./action.md) |
| `startTime`       | When the action started.                                                         | [`DateTime`](./date-time.md)                                           | [`Action`](./action.md) |
| `endTime`         | When the action ended.                                                           | [`DateTime`](./date-time.md)                                           | [`Action`](./action.md) |
| `error`           | An error produced by the action.                                                 | [`ThingVariant`](./thing-variant.md) \| [`String`](./string.md)        | [`Action`](./action.md) |
| `alternateNames`  | Alternate names (aliases) for the item.                                          | [`String`](./string.md)*                                               | [`Thing`](./thing.md)   |
| `description`     | A description of the item.                                                       | [`String`](./string.md)                                                | [`Thing`](./thing.md)   |
| `identifiers`     | Any kind of identifier for any kind of Thing.                                    | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*   | [`Thing`](./thing.md)   |
| `images`          | Images of the item.                                                              | [`ImageObject`](./image-object.md)*                                    | [`Thing`](./thing.md)   |
| `name`            | The name of the item.                                                            | [`String`](./string.md)                                                | [`Thing`](./thing.md)   |
| `url`             | The URL of the item.                                                             | [`String`](./string.md)                                                | [`Thing`](./thing.md)   |
| `id`              | The identifier for this item.                                                    | [`String`](./string.md)                                                | [`Entity`](./entity.md) |

# Related

The `ExecuteAction` type is related to these types:

- Parents: [`Action`](./action.md)
- Children: none

# Bindings

The `ExecuteAction` type is represented in:

- [JSON-LD](https://stencila.org/ExecuteAction.jsonld)
- [JSON Schema](https://stencila.org/ExecuteAction.schema.json)
- Python class [`ExecuteAction`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ExecuteAction`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execute_action.rs)
- TypeScript class [`ExecuteAction`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecuteAction.ts)

***

This documentation was generated from [`ExecuteAction.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecuteAction.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
