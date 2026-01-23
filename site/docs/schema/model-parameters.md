---
title: Model Parameters
description: Model selection and inference parameters for generative AI models.
---

# Properties

The `ModelParameters` type has these properties:

| Name              | Description                                             | Type                                       | Inherited from          |
| ----------------- | ------------------------------------------------------- | ------------------------------------------ | ----------------------- |
| `id`              | The identifier for this item.                           | [`String`](./string.md)                    | [`Entity`](./entity.md) |
| `modelIds`        | The ids of the models to select.                        | [`String`](./string.md)*                   | -                       |
| `replicates`      | The number of replicate inferences to run per model id. | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `qualityWeight`   | The relative weighting given to model quality (0-100).  | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `costWeight`      | The relative weighting given to model cost (0-100).     | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `speedWeight`     | The relative weighting given to model speed (0-100).    | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `minimumScore`    | The minimum score for models to be selected (0-100).    | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `temperature`     | The temperature option for model inference (0-100).     | [`UnsignedInteger`](./unsigned-integer.md) | -                       |
| `randomSeed`      | The random seed used for the model (if possible)        | [`Integer`](./integer.md)                  | -                       |
| `executeContent`  | Automatically execute generated content.                | [`Boolean`](./boolean.md)                  | -                       |
| `executionBounds` | The environment in which code should be executed.       | [`ExecutionBounds`](./execution-bounds.md) | -                       |
| `maximumRetries`  | When executing content, the maximum number of retries.  | [`UnsignedInteger`](./unsigned-integer.md) | -                       |

# Related

The `ModelParameters` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ModelParameters` type is represented in:

- [JSON-LD](https://stencila.org/ModelParameters.jsonld)
- [JSON Schema](https://stencila.org/ModelParameters.schema.json)
- Python class [`ModelParameters`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ModelParameters`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/model_parameters.rs)
- TypeScript class [`ModelParameters`](https://github.com/stencila/stencila/blob/main/ts/src/types/ModelParameters.ts)

***

This documentation was generated from [`ModelParameters.yaml`](https://github.com/stencila/stencila/blob/main/schema/ModelParameters.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
