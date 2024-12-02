# Duration

**A value that represents the difference between two timestamps.**

**`@id`**: [`schema:Duration`](https://schema.org/Duration)

## Properties

The `Duration` type has these properties:

| Name       | Aliases                  | `@id`                                      | Type                                                                                                 | Description                                | Inherited from                                                                                   |
| ---------- | ------------------------ | ------------------------------------------ | ---------------------------------------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `id`       | -                        | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)      | The identifier for this item.              | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `value`    | -                        | [`schema:value`](https://schema.org/value) | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)    | The time difference in `timeUnit`s.        | -                                                                                                |
| `timeUnit` | `time-unit`, `time_unit` | `stencila:timeUnit`                        | [`TimeUnit`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time-unit.md) | The time unit that the `value` represents. | -                                                                                                |

## Related

The `Duration` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Duration` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                                                                                                                             |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |                                                                                                                                   |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |                                                                                                                                   |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🟢 No loss    | 🟢 No loss  | 🚧 Under development | Encoded as [`<duration>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/duration.html) using special function |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | ⚠️ High loss |            | 🔶 Beta              |                                                                                                                                   |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |                                                                                                                                   |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |                                                                                                                                   |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |                                                                                                                                   |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |                                                                                                                                   |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                                                   |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |                                                                                                                                   |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |                                                                                                                                   |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                                                   |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                                                   |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                   |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                   |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                   |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                                                   |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                   |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                   |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                                                   |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                                                   |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |                                                                                                                                   |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |                                                                                                                                   |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |                                                                                                                                   |

## Bindings

The `Duration` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Duration.jsonld)
- [JSON Schema](https://stencila.org/Duration.schema.json)
- Python class [`Duration`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/duration.py)
- Rust struct [`Duration`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/duration.rs)
- TypeScript class [`Duration`](https://github.com/stencila/stencila/blob/main/ts/src/types/Duration.ts)

## Source

This documentation was generated from [`Duration.yaml`](https://github.com/stencila/stencila/blob/main/schema/Duration.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
