---
title: Model Parameters
description: Model selection and inference parameters for generative AI models.
---

# Properties

The `ModelParameters` type has these properties:

| Name              | Description                                             | Type                                       | Inherited from          | `JSON-LD @id`                        | Aliases                                                                        |
| ----------------- | ------------------------------------------------------- | ------------------------------------------ | ----------------------- | ------------------------------------ | ------------------------------------------------------------------------------ |
| `id`              | The identifier for this item.                           | [`String`](./string.md)                    | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id) | -                                                                              |
| `modelIds`        | The ids of the models to select.                        | [`String`](./string.md)*                   | -                       | `stencila:modelIds`                  | `models`, `model`, `model-ids`, `model_ids`, `modelId`, `model-id`, `model_id` |
| `replicates`      | The number of replicate inferences to run per model id. | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:replicates`                | `reps`                                                                         |
| `qualityWeight`   | The relative weighting given to model quality (0-100).  | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:qualityWeight`             | `quality`, `qual`, `quality-weight`, `quality_weight`                          |
| `costWeight`      | The relative weighting given to model cost (0-100).     | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:costWeight`                | `cost`, `cost-weight`, `cost_weight`                                           |
| `speedWeight`     | The relative weighting given to model speed (0-100).    | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:speedWeight`               | `speed`, `speed-weight`, `speed_weight`                                        |
| `minimumScore`    | The minimum score for models to be selected (0-100).    | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:minimumScore`              | `minimum-score`, `minimum_score`, `minScore`, `min-score`, `min_score`         |
| `temperature`     | The temperature option for model inference (0-100).     | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:temperature`               | `temp`                                                                         |
| `randomSeed`      | The random seed used for the model (if possible)        | [`Integer`](./integer.md)                  | -                       | `stencila:randomSeed`                | `random-seed`, `random_seed`, `rand-seed`, `rand_seed`, `seed`                 |
| `executeContent`  | Automatically execute generated content.                | [`Boolean`](./boolean.md)                  | -                       | `stencila:executeContent`            | `execute-content`, `execute_content`                                           |
| `executionBounds` | The environment in which code should be executed.       | [`ExecutionBounds`](./execution-bounds.md) | -                       | `stencila:executionBounds`           | `execution-bounds`, `execution_bounds`                                         |
| `maximumRetries`  | When executing content, the maximum number of retries.  | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:maximumRetries`            | `retries`, `maximum-retries`, `maximum_retries`                                |

# Related

The `ModelParameters` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `ModelParameters` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                    |
| [JATS](../formats/jats.md)                       |              |              |                                    |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                    |
| [CSV](../formats/csv.md)                         |              |              |                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                    |
| [Directory](../formats/directory.md)             |              |              |                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                    |

# Bindings

The `ModelParameters` type is represented in:

- [JSON-LD](https://stencila.org/ModelParameters.jsonld)
- [JSON Schema](https://stencila.org/ModelParameters.schema.json)
- Python class [`ModelParameters`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/model_parameters.py)
- Rust struct [`ModelParameters`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/model_parameters.rs)
- TypeScript class [`ModelParameters`](https://github.com/stencila/stencila/blob/main/ts/src/types/ModelParameters.ts)

# Source

This documentation was generated from [`ModelParameters.yaml`](https://github.com/stencila/stencila/blob/main/schema/ModelParameters.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
