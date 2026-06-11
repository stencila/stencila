---
title: Create Action
description: An action that creates a result.
---

This is an implementation of schema.org
[`CreateAction`](https://schema.org/CreateAction).

In Stencila provenance it represents creation of new document content,
generated outputs, or byte assets. It maps naturally to C2PA `c2pa.created`
when projecting provenance to standard C2PA actions.


# Analogues

The following external types, elements, or nodes are similar to a `CreateAction`:

- schema.org [`CreateAction`](https://schema.org/CreateAction): Direct schema.org source type.
- [C2PA c2pa.created action](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_actions): Approximate C2PA action projection when a manifest records newly created content.

# Properties

The `CreateAction` type has these properties:

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

The `CreateAction` type is related to these types:

- Parents: [`Action`](./action.md)
- Children: none

# Bindings

The `CreateAction` type is represented in:

- [JSON-LD](https://stencila.org/CreateAction.jsonld)
- [JSON Schema](https://stencila.org/CreateAction.schema.json)
- Python class [`CreateAction`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`CreateAction`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/create_action.rs)
- TypeScript class [`CreateAction`](https://github.com/stencila/stencila/blob/main/ts/src/types/CreateAction.ts)

***

This documentation was generated from [`CreateAction.yaml`](https://github.com/stencila/stencila/blob/main/schema/CreateAction.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
