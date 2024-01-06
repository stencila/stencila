# Styled Block

**Styled block content.**

**`@id`**: `stencila:StyledBlock`

This type is marked as unstable and is subject to change.

## Properties

The `StyledBlock` type has these properties:

| Name                | Aliases                                                                                                  | `@id`                                        | Type                                                                                                                   | Description                                                        | Inherited from                                                                                   |
| ------------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `id`                | -                                                                                                        | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                        | The identifier for this item.                                      | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `code`              | -                                                                                                        | `stencila:code`                              | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                            | The code of the equation in the `styleLanguage`.                   | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `styleLanguage`     | `style-language`, `style_language`                                                                       | `stencila:styleLanguage`                     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                        | The language used for the style specification e.g. css, tw         | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `authors`           | `author`                                                                                                 | [`schema:author`](https://schema.org/author) | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                      | The authors of the styling code.                                   | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `compilationDigest` | `compilation-digest`, `compilation_digest`                                                               | `stencila:compilationDigest`                 | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md) | A digest of the `code` and `styleLanguage`.                        | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `compilationErrors` | `compilation-errors`, `compilation_errors`, `compilationError`, `compilation-error`, `compilation_error` | `stencila:compilationErrors`                 | [`CompilationError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-error.md)*  | Errors generated when parsing and transpiling the style.           | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `css`               | -                                                                                                        | `stencila:css`                               | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                        | A Cascading Style Sheet (CSS) transpiled from the `code` property. | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `classes`           | `class`                                                                                                  | `stencila:classes`                           | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*                       | A list of class names associated with the node.                    | [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md) |
| `content`           | -                                                                                                        | `stencila:content`                           | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                        | The content within the styled block                                | -                                                                                                |

## Related

The `StyledBlock` type is related to these types:

- Parents: [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md)
- Children: none

## Formats

The `StyledBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes                                                                               |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |              | 🚧 Under development    | Encoded as [`<div>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |                  |              | 🚧 Under development    |                                                                                     |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | 🟢 No loss        | 🟢 No loss    | ⚠️ Alpha               | Encoded using special function                                                      |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                     |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss    | 🔶 Beta                 |                                                                                     |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |                                                                                     |

## Bindings

The `StyledBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.org/StyledBlock.jsonld)
- [JSON Schema](https://stencila.org/StyledBlock.schema.json)
- Python class [`StyledBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/styled_block.py)
- Rust struct [`StyledBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/styled_block.rs)
- TypeScript class [`StyledBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/StyledBlock.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `StyledBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property        | Complexity | Description                                                                      | Strategy                                      |
| --------------- | ---------- | -------------------------------------------------------------------------------- | --------------------------------------------- |
| `code`          | Min+       | Generate a simple fixed string of code.                                          | `Cord::new("code")`                           |
|                 | Low+       | Generate a random string of up to 10 alphanumeric & whitespace characters.       | `r"[a-zA-Z0-9 \t]{1,10}".prop_map(Cord::new)` |
|                 | High+      | Generate a random string of up to 100 characters (excluding control characters). | `r"[^\p{C}]{1,100}".prop_map(Cord::new)`      |
|                 | Max        | Generate an arbitrary string.                                                    | `String::arbitrary().prop_map(Cord::new)`     |
| `styleLanguage` | Min+       | Do not generate a style language.                                                | `None`                                        |
|                 | Low+       | Generate one of the well known style language short names.                       | `option::of(r"(css)\|(tw)")`                  |
|                 | High+      | Generate a random string of up to 10 alphanumeric characters.                    | `option::of(r"[a-zA-Z0-9]{1,10}")`            |
|                 | Max        | Generate an arbitrary string.                                                    | `option::of(String::arbitrary())`             |

## Source

This documentation was generated from [`StyledBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/StyledBlock.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.