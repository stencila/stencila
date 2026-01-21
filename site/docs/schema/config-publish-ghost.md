---
title: Config Publish Ghost
description: Ghost publishing options.
---

# Properties

The `ConfigPublishGhost` type has these properties:

| Name       | Description                                          | Type                                                         | Inherited from |
| ---------- | ---------------------------------------------------- | ------------------------------------------------------------ | -------------- |
| `slug`     | The URL slug for the page or post.                   | [`String`](./string.md)                                      | -              |
| `featured` | Whether the page or post is featured.                | [`Boolean`](./boolean.md)                                    | -              |
| `schedule` | The date that the page or post is to be published.   | [`Date`](./date.md)                                          | -              |
| `state`    | the state of the page or post eg draft or published. | [`ConfigPublishGhostState`](./config-publish-ghost-state.md) | -              |
| `tags`     | ghost tags.                                          | [`String`](./string.md)*                                     | -              |

# Related

The `ConfigPublishGhost` type is related to these types:

- Parents: None
- Children: none

# Bindings

The `ConfigPublishGhost` type is represented in:

- [JSON-LD](https://stencila.org/ConfigPublishGhost.jsonld)
- [JSON Schema](https://stencila.org/ConfigPublishGhost.schema.json)
- Python class [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_publish_ghost.py)
- Rust struct [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_publish_ghost.rs)
- TypeScript class [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigPublishGhost.ts)

# Source

This documentation was generated from [`ConfigPublishGhost.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigPublishGhost.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
