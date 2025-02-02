# Config Publish

**Publishing options.**

## Properties

The `ConfigPublish` type has these properties:

| Name     | Aliases | `@id` | Type                                                                                                                          | Description                | Inherited from |
| -------- | ------- | ----- | ----------------------------------------------------------------------------------------------------------------------------- | -------------------------- | -------------- |
| `ghost`  | -       | ``    | [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/config/config-publish-ghost.md)   | Ghost publishing options.  | -              |
| `zenodo` | -       | ``    | [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/config/config-publish-zenodo.md) | Zenodo publishing options. | -              |

## Related

The `ConfigPublish` type is related to these types:

- Parents: none
- Children: none

## Formats

The `ConfigPublish` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ConfigPublish` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ConfigPublish.jsonld)
- [JSON Schema](https://stencila.org/ConfigPublish.schema.json)
- Python class [`ConfigPublish`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_publish.py)
- Rust struct [`ConfigPublish`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_publish.rs)
- TypeScript class [`ConfigPublish`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigPublish.ts)

## Source

This documentation was generated from [`ConfigPublish.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigPublish.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
