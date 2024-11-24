# Provenance Count

**The count of the number of characters in a `ProvenanceCategory` within an entity.**

**`@id`**: `stencila:ProvenanceCount`

## Properties

The `ProvenanceCount` type has these properties:

| Name                 | Aliases                                      | `@id`                                | Type                                                                                                                      | Description                                                  | Inherited from                                                                                   |
| -------------------- | -------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `id`                 | -                                            | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The identifier for this item.                                | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `provenanceCategory` | `provenance-category`, `provenance_category` | `stencila:provenanceCategory`        | [`ProvenanceCategory`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-category.md) | The provenance category that the character count applies to. | -                                                                                                |
| `characterCount`     | `character-count`, `character_count`         | `stencila:characterCount`            | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md)        | The number of characters in the provenance category.         | -                                                                                                |
| `characterPercent`   | `character-percent`, `character_percent`     | `stencila:characterPercent`          | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md)        | The percentage of characters in the provenance category.     | -                                                                                                |

## Related

The `ProvenanceCount` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `ProvenanceCount` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🔶 Beta              |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | 🔶 Beta              |       |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |           | 🔶 Beta              |       |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |           | 🔶 Beta              |       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |           | 🔶 Beta              |       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |           | 🔶 Beta              |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | 🔶 Beta              |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |           | ⚠️ Alpha            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `ProvenanceCount` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ProvenanceCount.jsonld)
- [JSON Schema](https://stencila.org/ProvenanceCount.schema.json)
- Python class [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/provenance_count.py)
- Rust struct [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/provenance_count.rs)
- TypeScript class [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/ts/src/types/ProvenanceCount.ts)

## Source

This documentation was generated from [`ProvenanceCount.yaml`](https://github.com/stencila/stencila/blob/main/schema/ProvenanceCount.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
