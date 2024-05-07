# Modify Inline

**A suggestion to modify some inline content.**

**`@id`**: `stencila:ModifyInline`

## Properties

The `ModifyInline` type has these properties:

| Name               | Aliases                                  | `@id`                                | Type                                                                                                                  | Description                                                                           | Inherited from                                                                                                        |
| ------------------ | ---------------------------------------- | ------------------------------------ | --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `id`               | -                                        | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                       | The identifier for this item.                                                         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)                      |
| `suggestionStatus` | `suggestion-status`, `suggestion_status` | `stencila:suggestionStatus`          | [`SuggestionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-status.md) | The status of the suggestion including whether it is proposed, accepted, or rejected. | [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md)              |
| `content`          | -                                        | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)*                     | The content that is suggested to be inserted, modified, replaced, or deleted.         | [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-inline.md) |
| `operations`       | `operation`                              | `stencila:operations`                | [`ModifyOperation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-operation.md)*  | The operations to be applied to the nodes.                                            | -                                                                                                                     |

## Related

The `ModifyInline` type is related to these types:

- Parents: [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-inline.md)
- Children: none

## Formats

The `ModifyInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ModifyInline` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ModifyInline.jsonld)
- [JSON Schema](https://stencila.org/ModifyInline.schema.json)
- Python class [`ModifyInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/modify_inline.py)
- Rust struct [`ModifyInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/modify_inline.rs)
- TypeScript class [`ModifyInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/ModifyInline.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `ModifyInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

## Source

This documentation was generated from [`ModifyInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/ModifyInline.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
