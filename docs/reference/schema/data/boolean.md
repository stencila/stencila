# Boolean

**A value that is either true or false.**

**`@id`**: [`schema:Boolean`](https://schema.org/Boolean)

## Formats

The `Boolean` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding   | Decoding   | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ---------- | ---------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss  |            | 🔶 Beta              |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss |            | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🔷 Low loss |            | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | 🔷 Low loss | 🔷 Low loss | 🔶 Beta              |       |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | 🔷 Low loss | 🔷 Low loss | 🔶 Beta              |       |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | 🔷 Low loss | 🔷 Low loss | 🔶 Beta              |       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | 🔷 Low loss | 🔷 Low loss | 🔶 Beta              |       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | 🔷 Low loss | 🔷 Low loss | 🔶 Beta              |       |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss | 🔷 Low loss | 🚧 Under development |       |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss |            | 🚧 Under development |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | 🔷 Low loss |            | 🔶 Beta              |       |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss | 🔷 Low loss | 🚧 Under development |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss  | 🟢 No loss  | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss  | 🟢 No loss  | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss  | 🟢 No loss  | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss  | 🟢 No loss  | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss  | 🟢 No loss  | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss  | 🟢 No loss  | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss  | 🟢 No loss  | 🟢 Stable            |       |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss | 🔷 Low loss | 🚧 Under development |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |            |            | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |            |            | ⚠️ Alpha            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss |            | 🟢 Stable            |       |

## Bindings

The `Boolean` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Boolean.jsonld)
- [JSON Schema](https://stencila.org/Boolean.schema.json)
- Python type [`Boolean`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/boolean.py)
- Rust type [`Boolean`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/boolean.rs)
- TypeScript type [`Boolean`](https://github.com/stencila/stencila/blob/main/ts/src/types/Boolean.ts)

## Source

This documentation was generated from [`Boolean.yaml`](https://github.com/stencila/stencila/blob/main/schema/Boolean.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
