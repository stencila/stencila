# Styled Inline

**Styled inline content.**

**`@id`**: `stencila:StyledInline`

This type is marked as unstable and is subject to change.

## Properties

The `StyledInline` type has these properties:

| Name                  | Aliases                                                                                                            | `@id`                                        | Type                                                                                                                      | Description                                                            | Inherited from                                                                                   |
| --------------------- | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                  | -                                                                                                                  | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The identifier for this item.                                          | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `code`                | -                                                                                                                  | `stencila:code`                              | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                               | The code of the equation in the `styleLanguage`.                       | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `styleLanguage`       | `style-language`, `style_language`                                                                                 | `stencila:styleLanguage`                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The language used for the style specification e.g. css, tw             | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `authors`             | `author`                                                                                                           | [`schema:author`](https://schema.org/author) | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                         | The authors of the code and content in the styled node.                | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `provenance`          | -                                                                                                                  | `stencila:provenance`                        | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)*      | A summary of the provenance of the code and content in the styed node. | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `compilationDigest`   | `compilation-digest`, `compilation_digest`                                                                         | `stencila:compilationDigest`                 | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)    | A digest of the `code` and `styleLanguage`.                            | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `compilationMessages` | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` | `stencila:compilationMessages`               | [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-message.md)* | Messages generated while parsing and transpiling the style.            | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `css`                 | -                                                                                                                  | `stencila:css`                               | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | A Cascading Style Sheet (CSS) transpiled from the `code` property.     | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `classList`           | `class-list`, `class_list`                                                                                         | `stencila:classList`                         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | A space separated list of class names associated with the node.        | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `content`             | -                                                                                                                  | `stencila:content`                           | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)*                         | The content within the span.                                           | -                                                                                                |

## Related

The `StyledInline` type is related to these types:

- Parents: [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)
- Children: none

## Formats

The `StyledInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes                                                                                                                  |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |                                                                                                                        |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)                                  |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🟢 No loss    | 🟢 No loss | 🚧 Under development | Encoded as [`<styled-content>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/styled-content.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            | Encoded using implemented function                                                                                     |
| [MyST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)                | ⚠️ High loss |           | ⚠️ Alpha            |                                                                                                                        |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |                                                                                                                        |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                        |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                        |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                                                                                                        |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                        |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                        |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                        |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |                                                                                                                        |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |                                                                                                                        |

## Bindings

The `StyledInline` type is represented in these bindings:

- [JSON-LD](https://stencila.org/StyledInline.jsonld)
- [JSON Schema](https://stencila.org/StyledInline.schema.json)
- Python class [`StyledInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/styled_inline.py)
- Rust struct [`StyledInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/styled_inline.rs)
- TypeScript class [`StyledInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/StyledInline.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `StyledInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property        | Complexity | Description                                                                      | Strategy                                       |
| --------------- | ---------- | -------------------------------------------------------------------------------- | ---------------------------------------------- |
| `code`          | Min+       | Generate a simple fixed string of code.                                          | `Cord::from("code")`                           |
|                 | Low+       | Generate a random string of up to 10 alphanumeric & whitespace characters.       | `r"[a-zA-Z0-9 \t]{1,10}".prop_map(Cord::from)` |
|                 | High+      | Generate a random string of up to 100 characters (excluding control characters). | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`      |
|                 | Max        | Generate an arbitrary string.                                                    | `String::arbitrary().prop_map(Cord::from)`     |
| `styleLanguage` | Min+       | Do not generate a style language.                                                | `None`                                         |
|                 | Low+       | Generate one of the well known style language short names.                       | `option::of(r"(css)\|(tw)")`                   |
|                 | High+      | Generate a random string of up to 10 alphanumeric characters.                    | `option::of(r"[a-zA-Z0-9]{1,10}")`             |
|                 | Max        | Generate an arbitrary string.                                                    | `option::of(String::arbitrary())`              |
| `content`       | Min+       | Generate a single fixed text value.                                              | `vec![t("text")]`                              |
|                 | High+      | Generate up to two arbitrary, non-recursive, inline nodes                        | `vec_inlines_non_recursive(2)`                 |
|                 | Max        | Generate up to four arbitrary, non-recursive, inline nodes                       | `vec_inlines_non_recursive(4)`                 |

## Source

This documentation was generated from [`StyledInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/StyledInline.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
