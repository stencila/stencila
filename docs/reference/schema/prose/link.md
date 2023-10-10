# Link

**A hyperlink to other pages, sections within the same document, resources, or any URL.**

**`@id`**: `stencila:Link`

## Properties

The `Link` type has these properties:

| Name    | `@id`                                                            | Type                                                                                              | Description                                            | Inherited from                                                                                   |
| ------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| id      | [`schema:id`](https://schema.org/id)                             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item                           | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| content | `stencila:content`                                               | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The textual content of the link.                       | [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)     |
| target  | `stencila:target`                                                | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The target of the link.                                | [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)     |
| title   | [`schema:headline`](https://schema.org/headline)                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | A title for the link.                                  | [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)     |
| rel     | [`schema:linkRelationship`](https://schema.org/linkRelationship) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The relation between the target and the current thing. | [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)     |

## Related

The `Link` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Link` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                                                          |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | -------------------------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    | Encoded to tag [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)                            |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🔷 Low loss       |              | 🚧 Under development    | Encoded to tag [`<ext-link>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/ext-link.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🔷 Low loss       |              | 🚧 Under development    | Encoded using template `[{content}]({target})`                                                                 |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                                                |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                                |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                                                |

## Bindings

The `Link` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Link.jsonld)
- [JSON Schema](https://stencila.dev/Link.schema.json)
- Python class [`Link`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/link.py)
- Rust struct [`Link`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/link.rs)
- TypeScript class [`Link`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Link.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `Link` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any optional properties that are not in this table are set to `None`

| Property  | Complexity | Description                                                | Strategy                                                                |
| --------- | ---------- | ---------------------------------------------------------- | ----------------------------------------------------------------------- |
| `content` | Min+       | Generate a single fixed text value.                        | `vec(Just(Inline::Text(crate::Text::from("text"))), size_range(1..=1))` |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)`                                          |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)`                                          |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)`                                          |

## Source

This documentation was generated from [`Link.yaml`](https://github.com/stencila/stencila/blob/main/schema/Link.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).