---
title: Config Models
description: Model selection and execution options.
---

# Properties

The `ConfigModels` type has these properties:

| Name              | Description                                                            | Type                                       | Inherited from |
| ----------------- | ---------------------------------------------------------------------- | ------------------------------------------ | -------------- |
| `executeContent`  | Automatically execute generated content.                               | [`Boolean`](./boolean.md)                  | -              |
| `executionBounds` | The execution boundaries on model generated code.                      | [`ExecutionBounds`](./execution-bounds.md) | -              |
| `maximumRetries`  | When executing model generated content, the maximum number of retries. | [`Number`](./number.md)                    | -              |

# Related

The `ConfigModels` type is related to these types:

- Parents: None
- Children: none

# Bindings

The `ConfigModels` type is represented in:

- [JSON-LD](https://stencila.org/ConfigModels.jsonld)
- [JSON Schema](https://stencila.org/ConfigModels.schema.json)
- Python class [`ConfigModels`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ConfigModels`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_models.rs)
- TypeScript class [`ConfigModels`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigModels.ts)

***

This documentation was generated from [`ConfigModels.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigModels.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
