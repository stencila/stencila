# File

**A file on the file system.**

Previously this type extended `CreativeWork`.
However, to avoid consuming more memory than necessary when creating directory listings
with many files, it now extends `Entity`.


**`@id`**: `stencila:File`

## Properties

The `File` type has these properties:

| Name        | Aliases                                      | `@id`                                                        | Type                                                                                            | Description                                                    | Inherited from                                                                                   |
| ----------- | -------------------------------------------- | ------------------------------------------------------------ | ----------------------------------------------------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`        | -                                            | [`schema:id`](https://schema.org/id)                         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.                                  | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `name`      | -                                            | [`schema:name`](https://schema.org/name)                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The name of the file.                                          | -                                                                                                |
| `path`      | -                                            | `stencila:path`                                              | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The path (absolute or relative) of the file on the file system | -                                                                                                |
| `mediaType` | `encodingFormat`, `media-type`, `media_type` | [`schema:encodingFormat`](https://schema.org/encodingFormat) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | IANA media type (MIME type).                                   | -                                                                                                |

## Related

The `File` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `File` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `File` type is represented in these bindings:

- [JSON-LD](https://stencila.org/File.jsonld)
- [JSON Schema](https://stencila.org/File.schema.json)
- Python class [`File`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/file.py)
- Rust struct [`File`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/file.rs)
- TypeScript class [`File`](https://github.com/stencila/stencila/blob/main/ts/src/types/File.ts)

## Source

This documentation was generated from [`File.yaml`](https://github.com/stencila/stencila/blob/main/schema/File.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).