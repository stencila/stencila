# Emphasis

**Emphasized content.**

Analogues of `Delete` in other schema include:
  - HTML [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)
  - JATS XML [`<italic>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/italic.html)
  - MDAST [`Emphasis`](https://github.com/syntax-tree/mdast#emphasis)
  - Pandoc [`Emph`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L256)


**`@id`**: `stencila:Emphasis`

## Properties

The `Emphasis` type has these properties:

| Name      | Aliases | `@id`                                | Type                                                                                              | Description                   | Inherited from                                                                                   |
| --------- | ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`      | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item. | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `content` | -       | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The content that is marked.   | [`Mark`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/mark.md)     |

## Related

The `Emphasis` type is related to these types:

- Parents: [`Mark`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/mark.md)
- Children: none

## Formats

The `Emphasis` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                                                                                                  |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ------------------------------------------------------------------------------------------------------ |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |                                                                                                        |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🟢 No loss    |            | 🚧 Under development | Encoded as [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)                      |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🟢 No loss    | 🟢 No loss  | 🚧 Under development | Encoded as [`<italic>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/italic.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              | Encoded as `_{{content}}_`                                                                             |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                        |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                        |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                        |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                        |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                        |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |                                                                                                        |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |                                                                                                        |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                        |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                        |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                        |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                        |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                        |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                                        |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                        |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                        |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                                        |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                                        |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |                                                                                                        |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |                                                                                                        |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |                                                                                                        |

## Bindings

The `Emphasis` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Emphasis.jsonld)
- [JSON Schema](https://stencila.org/Emphasis.schema.json)
- Python class [`Emphasis`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/emphasis.py)
- Rust struct [`Emphasis`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/emphasis.rs)
- TypeScript class [`Emphasis`](https://github.com/stencila/stencila/blob/main/ts/src/types/Emphasis.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `Emphasis` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

## Source

This documentation was generated from [`Emphasis.yaml`](https://github.com/stencila/stencila/blob/main/schema/Emphasis.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
