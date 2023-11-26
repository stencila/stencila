# Modify Block

**A suggestion to modify some block content.**

**`@id`**: `stencila:ModifyBlock`

## Properties

The `ModifyBlock` type has these properties:

| Name         | Aliases     | `@id`                                | Type                                                                                                                 | Description                                              | Inherited from                                                                                                      |
| ------------ | ----------- | ------------------------------------ | -------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `id`         | -           | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                      | The identifier for this item.                            | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)                    |
| `content`    | -           | `stencila:content`                   | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                      | The content that is suggested to be inserted or deleted. | [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-block.md) |
| `operations` | `operation` | `stencila:operations`                | [`ModifyOperation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-operation.md)* | The operations to be applied to the nodes.               | [`Modify`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify.md)                    |

## Related

The `ModifyBlock` type is related to these types:

- Parents: [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-block.md)[`Modify`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify.md)
- Children: none

## Formats

The `ModifyBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes                          |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ------------------------------ |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |              | 🚧 Under development    |                                |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |                  |              | 🚧 Under development    |                                |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss     |              | ⚠️ Alpha               | Encoded using special function |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |                                |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss    | 🔶 Beta                 |                                |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |                                |

## Bindings

The `ModifyBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ModifyBlock.jsonld)
- [JSON Schema](https://stencila.org/ModifyBlock.schema.json)
- Python class [`ModifyBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/modify_block.py)
- Rust struct [`ModifyBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/modify_block.rs)
- TypeScript class [`ModifyBlock`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ModifyBlock.ts)

## Source

This documentation was generated from [`ModifyBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/ModifyBlock.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).