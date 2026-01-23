---
title: Config
description: Stencila document configuration options.
---

# Properties

The `Config` type has these properties:

| Name      | Description                                                        | Type                                   | Inherited from |
| --------- | ------------------------------------------------------------------ | -------------------------------------- | -------------- |
| `theme`   | The styling theme for the document                                 | [`String`](./string.md)                | -              |
| `models`  | The parameters used for selecting and running generative AI models | [`ConfigModels`](./config-models.md)   | -              |
| `publish` | Publishing configuration options                                   | [`ConfigPublish`](./config-publish.md) | -              |

# Related

The `Config` type is related to these types:

- Parents: None
- Children: none

# Bindings

The `Config` type is represented in:

- [JSON-LD](https://stencila.org/Config.jsonld)
- [JSON Schema](https://stencila.org/Config.schema.json)
- Python class [`Config`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Config`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config.rs)
- TypeScript class [`Config`](https://github.com/stencila/stencila/blob/main/ts/src/types/Config.ts)

***

This documentation was generated from [`Config.yaml`](https://github.com/stencila/stencila/blob/main/schema/Config.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
