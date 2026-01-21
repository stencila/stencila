---
title: Config Publish Zenodo
description: Zenodo publishing options.
---

# Properties

The `ConfigPublishZenodo` type has these properties:

| Name           | Description                       | Type                                                                        | Inherited from |
| -------------- | --------------------------------- | --------------------------------------------------------------------------- | -------------- |
| `embargoed`    | The date of embargoed.            | [`Date`](./date.md)                                                         | -              |
| `access_right` | The access right of the document. | [`ConfigPublishZenodoAccessRight`](./config-publish-zenodo-access-right.md) | -              |
| `notes`        | extra notes about deposition.     | [`String`](./string.md)                                                     | -              |
| `method`       | The methodology of the study.     | [`String`](./string.md)                                                     | -              |

# Related

The `ConfigPublishZenodo` type is related to these types:

- Parents: None
- Children: none

# Bindings

The `ConfigPublishZenodo` type is represented in:

- [JSON-LD](https://stencila.org/ConfigPublishZenodo.jsonld)
- [JSON Schema](https://stencila.org/ConfigPublishZenodo.schema.json)
- Python class [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_publish_zenodo.py)
- Rust struct [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_publish_zenodo.rs)
- TypeScript class [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigPublishZenodo.ts)

# Source

This documentation was generated from [`ConfigPublishZenodo.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigPublishZenodo.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
