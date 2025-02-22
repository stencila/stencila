---
title: Model Parameters
description: Model selection and inference parameters for generative AI models.
config:
  publish:
    ghost:
      type: post
      slug: model-parameters
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

# Properties

The `ModelParameters` type has these properties:

| Name              | Description                                             | Type                                                                                  | Inherited from                                                     | `JSON-LD @id`                        | Aliases                                                                        |
| ----------------- | ------------------------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ------------------------------------------------------------------------------ |
| `id`              | The identifier for this item.                           | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                                                                              |
| `modelIds`        | The ids of the models to select.                        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                   | -                                                                  | `stencila:modelIds`                  | `models`, `model`, `model-ids`, `model_ids`, `modelId`, `model-id`, `model_id` |
| `replicates`      | The number of replicate inferences to run per model id. | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:replicates`                | `reps`                                                                         |
| `qualityWeight`   | The relative weighting given to model quality (0-100).  | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:qualityWeight`             | `quality`, `qual`, `quality-weight`, `quality_weight`                          |
| `costWeight`      | The relative weighting given to model cost (0-100).     | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:costWeight`                | `cost`, `cost-weight`, `cost_weight`                                           |
| `speedWeight`     | The relative weighting given to model speed (0-100).    | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:speedWeight`               | `speed`, `speed-weight`, `speed_weight`                                        |
| `minimumScore`    | The minimum score for models to be selected (0-100).    | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:minimumScore`              | `minimum-score`, `minimum_score`, `minScore`, `min-score`, `min_score`         |
| `temperature`     | The temperature option for model inference (0-100).     | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:temperature`               | `temp`                                                                         |
| `randomSeed`      | The random seed used for the model (if possible)        | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                  | -                                                                  | `stencila:randomSeed`                | `random-seed`, `random_seed`, `rand-seed`, `rand_seed`, `seed`                 |
| `executeContent`  | Automatically execute generated content.                | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)                  | -                                                                  | `stencila:executeContent`            | `execute-content`, `execute_content`                                           |
| `executionBounds` | The environment in which code should be executed.       | [`ExecutionBounds`](https://stencila.ghost.io/docs/reference/schema/execution-bounds) | -                                                                  | `stencila:executionBounds`           | `execution-bounds`, `execution_bounds`                                         |
| `maximumRetries`  | When executing content, the maximum number of retries.  | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:maximumRetries`            | `retries`, `maximum-retries`, `maximum_retries`                                |

# Related

The `ModelParameters` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `ModelParameters` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support                            | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                |              |            |                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | ⚠️ High loss |            | Encoded using implemented function |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | ⚠️ High loss |            |                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | ⚠️ High loss |            |                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | ⚠️ High loss |            |                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | ⚠️ High loss |            |                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |                                    |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |                                    |
| [CBOR+Zstandard](https://stencila.ghost.io/docs/reference/formats/cbor.zstd) | 🟢 No loss    | 🟢 No loss  |                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |                                    |

# Bindings

The `ModelParameters` type is represented in:

- [JSON-LD](https://stencila.org/ModelParameters.jsonld)
- [JSON Schema](https://stencila.org/ModelParameters.schema.json)
- Python class [`ModelParameters`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/model_parameters.py)
- Rust struct [`ModelParameters`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/model_parameters.rs)
- TypeScript class [`ModelParameters`](https://github.com/stencila/stencila/blob/main/ts/src/types/ModelParameters.ts)

# Source

This documentation was generated from [`ModelParameters.yaml`](https://github.com/stencila/stencila/blob/main/schema/ModelParameters.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
