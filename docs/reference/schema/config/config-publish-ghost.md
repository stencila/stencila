# Config Publish Ghost

**Ghost publishing options.**

## Properties

The `ConfigPublishGhost` type has these properties:

| Name       | Aliases | `@id` | Type                                                                                              | Description                                        | Inherited from |
| ---------- | ------- | ----- | ------------------------------------------------------------------------------------------------- | -------------------------------------------------- | -------------- |
| `slug`     | -       | ``    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The URL slug for the page or post.                 | -              |
| `featured` | -       | ``    | [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md) | Whether the page or post is featured.              | -              |
| `schedule` | -       | ``    | [`Date`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date.md)       | The date that the page or post is to be published. | -              |

## Related

The `ConfigPublishGhost` type is related to these types:

- Parents: none
- Children: none

## Formats

The `ConfigPublishGhost` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ConfigPublishGhost` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ConfigPublishGhost.jsonld)
- [JSON Schema](https://stencila.org/ConfigPublishGhost.schema.json)
- Python class [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_publish_ghost.py)
- Rust struct [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_publish_ghost.rs)
- TypeScript class [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigPublishGhost.ts)

## Source

This documentation was generated from [`ConfigPublishGhost.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigPublishGhost.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
