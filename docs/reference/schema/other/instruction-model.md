# Instruction Model

**Model selection criteria and execution options for the generative model used for an instruction.**

**`@id`**: `stencila:InstructionModel`

## Properties

The `InstructionModel` type has these properties:

| Name            | Aliases                            | `@id`                                | Type                                                                                                               | Description                                            | Inherited from                                                                                   |
| --------------- | ---------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `id`            | -                                  | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The identifier for this item.                          | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `idPattern`     | `id-pattern`, `id_pattern`         | `stencila:idPattern`                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | A pattern to filter model ids by.                      | -                                                                                                |
| `qualityWeight` | `quality-weight`, `quality_weight` | `stencila:qualityWeight`             | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The relative weighting given to model quality (0-100). | -                                                                                                |
| `speedWeight`   | `speed-weight`, `speed_weight`     | `stencila:speedWeight`               | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The relative weighting given to model speed (0-100).   | -                                                                                                |
| `costWeight`    | `cost-weight`, `cost_weight`       | `stencila:costWeight`                | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The relative weighting given to model cost (0-100).    | -                                                                                                |
| `minimumScore`  | `minimum-score`, `minimum_score`   | `stencila:minimumScore`              | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The minimum score for models to be selected (0-100).   | -                                                                                                |
| `temperature`   | -                                  | `stencila:temperature`               | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The temperature option for model inference (0-100).    | -                                                                                                |
| `randomSeed`    | `random-seed`, `random_seed`       | `stencila:randomSeed`                | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                  | The random seed used for the model (if possible)       | -                                                                                                |

## Related

The `InstructionModel` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `InstructionModel` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |            | 🔶 Beta              |       |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |       |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |       |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |       |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |       |

## Bindings

The `InstructionModel` type is represented in these bindings:

- [JSON-LD](https://stencila.org/InstructionModel.jsonld)
- [JSON Schema](https://stencila.org/InstructionModel.schema.json)
- Python class [`InstructionModel`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/instruction_model.py)
- Rust struct [`InstructionModel`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruction_model.rs)
- TypeScript class [`InstructionModel`](https://github.com/stencila/stencila/blob/main/ts/src/types/InstructionModel.ts)

## Source

This documentation was generated from [`InstructionModel.yaml`](https://github.com/stencila/stencila/blob/main/schema/InstructionModel.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
