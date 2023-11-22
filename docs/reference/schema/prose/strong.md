# Strong

**Strongly emphasized content.**

**`@id`**: `stencila:Strong`

## Properties

The `Strong` type has these properties:

| Name      | Aliases | `@id`                                | Type                                                                                              | Description                   | Inherited from                                                                                   |
| --------- | ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`      | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item. | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `content` | -       | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The content that is marked.   | [`Mark`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/mark.md)     |

## Related

The `Strong` type is related to these types:

- Parents: [`Mark`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/mark.md)
- Children: none

## Formats

The `Strong` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes                                                                                              |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | -------------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🟢 No loss        |              | 🚧 Under development    | Encoded as [`<strong>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong)          |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              | 🟢 No loss        | 🟢 No loss    | 🚧 Under development    | Encoded as [`<bold>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/bold.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | 🟢 No loss        | 🟢 No loss    | ⚠️ Alpha               | Encoded as `**{content}**`                                                                         |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |                                                                                                    |

## Bindings

The `Strong` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Strong.jsonld)
- [JSON Schema](https://stencila.dev/Strong.schema.json)
- Python class [`Strong`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/strong.py)
- Rust struct [`Strong`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/strong.rs)
- TypeScript class [`Strong`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Strong.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `Strong` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

## Source

This documentation was generated from [`Strong.yaml`](https://github.com/stencila/stencila/blob/main/schema/Strong.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.