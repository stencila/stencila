---
title: Convert Action
description: An action that converts a resource from one representation or format to another.
---

This is a Stencila extension of schema.org [`Action`](./action.md).

It represents format, encoding, packaging, or document-model conversion. In
Stencila, rendering an executable document can be represented as an
[`ExecuteAction`](./execute-action.md) followed by a `ConvertAction`, rather
than as a separate render-specific action. In graphs, conversion actions are
usually attached to the resource-flow edge that links the source to the
converted result.


# Analogues

The following external types, elements, or nodes are similar to a `ConvertAction`:

- schema.org [`Action`](https://schema.org/Action): Base schema.org action type extended for conversion-specific provenance.
- [C2PA c2pa.converted action](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_actions): Close C2PA action projection when the file format or representation changed.
- [PROV wasDerivedFrom](https://www.w3.org/TR/prov-o/#wasDerivedFrom): Approximate provenance analogue for conversion-derived results.

# Properties

The `ConvertAction` type has these properties:

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

The `ConvertAction` type is related to these types:

- Parents: [`Action`](./action.md)
- Children: none

# Bindings

The `ConvertAction` type is represented in:

- [JSON-LD](https://stencila.org/ConvertAction.jsonld)
- [JSON Schema](https://stencila.org/ConvertAction.schema.json)
- Python class [`ConvertAction`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ConvertAction`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/convert_action.rs)
- TypeScript class [`ConvertAction`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConvertAction.ts)

***

This documentation was generated from [`ConvertAction.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConvertAction.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
