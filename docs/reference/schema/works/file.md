# File

**A file on the file system.**

Previously this type extended `CreativeWork`.
However, to avoid consuming more memory than necessary when creating directory listings
with many files, it now extends `Entity`.


**`@id`**: `stencila:File`

## Properties

The `File` type has these properties:

| Name               | Aliases                                      | `@id`                                                        | Type                                                                                                               | Description                                                    | Inherited from                                                                                   |
| ------------------ | -------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`               | -                                            | [`schema:id`](https://schema.org/id)                         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The identifier for this item.                                  | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `name`             | -                                            | [`schema:name`](https://schema.org/name)                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The name of the file.                                          | -                                                                                                |
| `path`             | -                                            | `stencila:path`                                              | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The path (absolute or relative) of the file on the file system | -                                                                                                |
| `mediaType`        | `encodingFormat`, `media-type`, `media_type` | [`schema:encodingFormat`](https://schema.org/encodingFormat) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | IANA media type (MIME type).                                   | -                                                                                                |
| `transferEncoding` | `transfer-encoding`, `transfer_encoding`     | `stencila:transferEncoding`                                  | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The encoding used for the context (e.g. base64, gz)            | -                                                                                                |
| `size`             | -                                            | [`schema:size`](https://schema.org/size)                     | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The size of the content in bytes                               | -                                                                                                |
| `content`          | -                                            | [`schema:content`](https://schema.org/content)               | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The content of the file.                                       | -                                                                                                |

## Related

The `File` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `File` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | ⚠️ High loss |            | 🔶 Beta              |       |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |       |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |       |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |       |
| [IPYNB](https://github.com/stencila/stencila/blob/main/docs/reference/formats/ipynb.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [TeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/tex.md)                  | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |       |
| [Lexical JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/lexical.md)     | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |       |
| [Koenig JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/koenig.md)       | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |       |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |       |

## Bindings

The `File` type is represented in these bindings:

- [JSON-LD](https://stencila.org/File.jsonld)
- [JSON Schema](https://stencila.org/File.schema.json)
- Python class [`File`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/file.py)
- Rust struct [`File`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/file.rs)
- TypeScript class [`File`](https://github.com/stencila/stencila/blob/main/ts/src/types/File.ts)

## Source

This documentation was generated from [`File.yaml`](https://github.com/stencila/stencila/blob/main/schema/File.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
