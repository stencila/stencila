# Modify Block

**A suggestion to modify some block content.**

**`@id`**: `stencila:ModifyBlock`

## Properties

The `ModifyBlock` type has these properties:

| Name         | Aliases     | `@id`                                | Type                                                                                                                 | Description                                                                   | Inherited from                                                                                                      |
| ------------ | ----------- | ------------------------------------ | -------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `id`         | -           | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                      | The identifier for this item.                                                 | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)                    |
| `content`    | -           | `stencila:content`                   | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                      | The content that is suggested to be inserted, modified, replaced, or deleted. | [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-block.md) |
| `operations` | `operation` | `stencila:operations`                | [`ModifyOperation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-operation.md)* | The operations to be applied to the nodes.                                    | -                                                                                                                   |

## Related

The `ModifyBlock` type is related to these types:

- Parents: [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-block.md)
- Children: none

## Formats

The `ModifyBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            | Encoded using implemented function |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |                                    |

## Bindings

The `ModifyBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ModifyBlock.jsonld)
- [JSON Schema](https://stencila.org/ModifyBlock.schema.json)
- Python class [`ModifyBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/modify_block.py)
- Rust struct [`ModifyBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/modify_block.rs)
- TypeScript class [`ModifyBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/ModifyBlock.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `ModifyBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                               | Strategy                      |
| --------- | ---------- | --------------------------------------------------------- | ----------------------------- |
| `content` | Min+       | Generate a single fixed paragraph.                        | `vec![p([t("text")])]`        |
|           | Low+       | Generate a single arbitrary, non-recursive, block node    | `vec_blocks_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, block nodes  | `vec_blocks_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, block nodes | `vec_blocks_non_recursive(4)` |

## Source

This documentation was generated from [`ModifyBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/ModifyBlock.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.