# Execution Tag

**A tag on code that affects its execution.**

**`@id`**: `stencila:ExecutionTag`

## Properties

The `ExecutionTag` type has these properties:

| Name       | Aliases                  | `@id`                                      | Type                                                                                              | Description                               | Inherited from                                                                                   |
| ---------- | ------------------------ | ------------------------------------------ | ------------------------------------------------------------------------------------------------- | ----------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`       | -                        | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item.             | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `name`     | -                        | [`schema:name`](https://schema.org/name)   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The name of the tag                       | -                                                                                                |
| `value`    | -                        | [`schema:value`](https://schema.org/value) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The value of the tag                      | -                                                                                                |
| `isGlobal` | `is-global`, `is_global` | `stencila:isGlobal`                        | [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md) | Whether the tag is global to the document | -                                                                                                |

## Related

The `ExecutionTag` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `ExecutionTag` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.md)           | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)    |              |           | 🚧 Under development |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `ExecutionTag` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ExecutionTag.jsonld)
- [JSON Schema](https://stencila.org/ExecutionTag.schema.json)
- Python class [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_tag.py)
- Rust struct [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_tag.rs)
- TypeScript class [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionTag.ts)

## Source

This documentation was generated from [`ExecutionTag.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionTag.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).