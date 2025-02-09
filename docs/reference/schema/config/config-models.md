# Config Models

**Model selection and execution options.**

## Properties

The `ConfigModels` type has these properties:

| Name              | Aliases                                                                             | `@id` | Type                                                                                                               | Description                                                            | Inherited from |
| ----------------- | ----------------------------------------------------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------- | -------------- |
| `executeContent`  | `execute-content`, `execute_content`                                                | ``    | [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)                  | Automatically execute generated content.                               | -              |
| `executionBounds` | `execution-bounds`, `execution_bounds`                                              | ``    | [`ExecutionBounds`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-bounds.md) | The execution boundaries on model generated code.                      | -              |
| `maximumRetries`  | `max-retries`, `maximum-retries`, `execution-retries`, `retries`, `maximum_retries` | ``    | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md)                    | When executing model generated content, the maximum number of retries. | -              |

## Related

The `ConfigModels` type is related to these types:

- Parents: none
- Children: none

## Formats

The `ConfigModels` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding | Decoding | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | -------- | -------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        |          |          | 🔶 Beta              |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                |          |          | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |          |          | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              |          |          | 🔶 Beta              |       |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    |          |          | 🔶 Beta              |       |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      |          |          | 🔶 Beta              |       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       |          |          | 🔶 Beta              |       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        |          |          | 🔶 Beta              |       |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              |          |          | 🚧 Under development |       |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  |          |          | 🚧 Under development |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          |          |          | 🔶 Beta              |       |
| [IPYNB](https://github.com/stencila/stencila/blob/main/docs/reference/formats/ipynb.md)              |          |          | 🚧 Under development |       |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) |          |          | 🚧 Under development |       |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     |          |          | 🚧 Under development |       |
| [TeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/tex.md)                  |          |          | 🚧 Under development |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                |          |          | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        |          |          | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              |          |          | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           |          |          | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                |          |          | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) |          |          | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                |          |          | 🟢 Stable            |       |
| [Lexical JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/lexical.md)     |          |          | ⚠️ Alpha            |       |
| [Koenig JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/koenig.md)       |          |          | ⚠️ Alpha            |       |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        |          |          | 🚧 Under development |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |          |          | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |          |          | ⚠️ Alpha            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              |          |          | 🟢 Stable            |       |

## Bindings

The `ConfigModels` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ConfigModels.jsonld)
- [JSON Schema](https://stencila.org/ConfigModels.schema.json)
- Python class [`ConfigModels`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_models.py)
- Rust struct [`ConfigModels`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_models.rs)
- TypeScript class [`ConfigModels`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigModels.ts)

## Source

This documentation was generated from [`ConfigModels.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigModels.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
