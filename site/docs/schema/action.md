---
title: Action
description: An action performed by an agent.
---

This is an implementation of schema.org
[`Action`](https://schema.org/Action).

In Stencila Schema it provides a neutral activity/event record for
provenance. Actions may be used as standalone nodes in document models or
attached to `GraphEdge.actions` when a concrete activity explains a
resource-flow relationship. Actions retain schema.org metadata such as the
agent, object, result, instrument, status, and timing.

The schema.org `object` and `result` properties are exposed as the plural
Stencila properties `objects` and `results` because an action can consume or
produce several resources or literal values. The singular schema.org property
names are retained as aliases.

More specific action types such as [`CreateAction`](./create-action.md),
[`ExecuteAction`](./execute-action.md), and
[`ConvertAction`](./convert-action.md) should be used when the activity kind
is known.


# Analogues

The following external types, elements, or nodes are similar to a `Action`:

- schema.org [`Action`](https://schema.org/Action): Direct schema.org source type, used as the common base for provenance activities.

# Properties

The `Action` type has these properties:

| Name              | Description                                                                      | Type                                                                   | Inherited from          |
| ----------------- | -------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ----------------------- |
| `actionStatus`    | The current status of the action.                                                | [`ActionStatusType`](./action-status-type.md)                          | -                       |
| `agent`           | The direct performer or driver of the action.                                    | [`ActionAgent`](./action-agent.md)                                     | -                       |
| `participants`    | Other agents that participated in the action.                                    | [`ActionAgent`](./action-agent.md)*                                    | -                       |
| `provider`        | The service provider, service operator, or performer responsible for the action. | [`ActionAgent`](./action-agent.md)                                     | -                       |
| `objects`         | The objects or input values upon which the action is carried out.                | [`Node`](./node.md)*                                                   | -                       |
| `results`         | The objects or values produced by the action.                                    | [`Node`](./node.md)*                                                   | -                       |
| `instrument`      | The object, software, or other instrument that helped perform the action.        | [`ThingVariant`](./thing-variant.md) \| [`String`](./string.md)        | -                       |
| `environment`     | Environment variables or settings that affected the action.                      | [`PropertyValue`](./property-value.md)*                                | -                       |
| `containerImages` | Container images used by the action.                                             | ([`ContainerImage`](./container-image.md) \| [`String`](./string.md))* | -                       |
| `startTime`       | When the action started.                                                         | [`DateTime`](./date-time.md)                                           | -                       |
| `endTime`         | When the action ended.                                                           | [`DateTime`](./date-time.md)                                           | -                       |
| `error`           | An error produced by the action.                                                 | [`ThingVariant`](./thing-variant.md) \| [`String`](./string.md)        | -                       |
| `alternateNames`  | Alternate names (aliases) for the item.                                          | [`String`](./string.md)*                                               | [`Thing`](./thing.md)   |
| `description`     | A description of the item.                                                       | [`String`](./string.md)                                                | [`Thing`](./thing.md)   |
| `identifiers`     | Any kind of identifier for any kind of Thing.                                    | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))*   | [`Thing`](./thing.md)   |
| `images`          | Images of the item.                                                              | [`ImageObject`](./image-object.md)*                                    | [`Thing`](./thing.md)   |
| `name`            | The name of the item.                                                            | [`String`](./string.md)                                                | [`Thing`](./thing.md)   |
| `url`             | The URL of the item.                                                             | [`String`](./string.md)                                                | [`Thing`](./thing.md)   |
| `id`              | The identifier for this item.                                                    | [`String`](./string.md)                                                | [`Entity`](./entity.md) |

# Related

The `Action` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: [`ConvertAction`](./convert-action.md), [`CreateAction`](./create-action.md), [`ExecuteAction`](./execute-action.md)

# Bindings

The `Action` type is represented in:

- [JSON-LD](https://stencila.org/Action.jsonld)
- [JSON Schema](https://stencila.org/Action.schema.json)
- Python class [`Action`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Action`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/action.rs)
- TypeScript class [`Action`](https://github.com/stencila/stencila/blob/main/ts/src/types/Action.ts)

***

This documentation was generated from [`Action.yaml`](https://github.com/stencila/stencila/blob/main/schema/Action.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
