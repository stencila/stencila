---
title: Container Image
description: A container image used by a computational action.
---

This type records lightweight identity for container images used during
computation. It follows the Workflow Run RO-Crate pattern for container
images, where an action can point to an image with registry, name, tag, and
digest metadata.

Key properties include `name`, `additionalType`, `registry`, `tag`, and
`sha256`.


# Analogues

The following external types, elements, or nodes are similar to a `ContainerImage`:

- [Workflow Run RO-Crate ContainerImage](https://www.researchobject.org/workflow-run-crate/profiles/process_run_crate/): Prior-art type for recording container images used by workflow or process run actions.

# Properties

The `ContainerImage` type has these properties:

| Name             | Description                                               | Type                                                                 | Inherited from          |
| ---------------- | --------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `additionalType` | The specific image type, such as DockerImage or SIFImage. | [`String`](./string.md)                                              | -                       |
| `registry`       | The registry or service that hosts the image.             | [`String`](./string.md)                                              | -                       |
| `tag`            | The image tag.                                            | [`String`](./string.md)                                              | -                       |
| `sha256`         | The SHA-256 checksum of the image.                        | [`String`](./string.md)                                              | -                       |
| `alternateNames` | Alternate names (aliases) for the item.                   | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                                | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing.             | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                                       | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                                      | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `id`             | The identifier for this item.                             | [`String`](./string.md)                                              | [`Entity`](./entity.md) |

# Related

The `ContainerImage` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `ContainerImage` type is represented in:

- [JSON-LD](https://stencila.org/ContainerImage.jsonld)
- [JSON Schema](https://stencila.org/ContainerImage.schema.json)
- Python class [`ContainerImage`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ContainerImage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/container_image.rs)
- TypeScript class [`ContainerImage`](https://github.com/stencila/stencila/blob/main/ts/src/types/ContainerImage.ts)

***

This documentation was generated from [`ContainerImage.yaml`](https://github.com/stencila/stencila/blob/main/schema/ContainerImage.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
