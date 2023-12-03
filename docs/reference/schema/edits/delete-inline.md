# Delete Inline

**A suggestion to delete some inline content.**

**`@id`**: `stencila:DeleteInline`

## Properties

The `DeleteInline` type has these properties:

| Name      | Aliases | `@id`                                | Type                                                                                              | Description                                              | Inherited from                                                                                                        |
| --------- | ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `id`      | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item.                            | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)                      |
| `content` | -       | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The content that is suggested to be inserted or deleted. | [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-inline.md) |

## Related

The `DeleteInline` type is related to these types:

- Parents: [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-inline.md)
- Children: none

## Formats

The `DeleteInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes                                                                               |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |              | 🚧 Under development    | Encoded as [`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              | ⚠️ High loss     |              | 🚧 Under development    |                                                                                     |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss     |              | ⚠️ Alpha               | Encoded as `{{--{content}--}}`                                                      |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                     |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss    | 🔶 Beta                 |                                                                                     |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |                                                                                     |

## Bindings

The `DeleteInline` type is represented in these bindings:

- [JSON-LD](https://stencila.org/DeleteInline.jsonld)
- [JSON Schema](https://stencila.org/DeleteInline.schema.json)
- Python class [`DeleteInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/delete_inline.py)
- Rust struct [`DeleteInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/delete_inline.rs)
- TypeScript class [`DeleteInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/DeleteInline.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `DeleteInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

## Source

This documentation was generated from [`DeleteInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/DeleteInline.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.