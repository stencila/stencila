# Raw Block

**Document content in a specific format**

The content of the block is not decoded by any codecs and is output when the encoding format
matches that of the raw block and the `render` option is used.
Analogous to node types in [Pandoc](https://github.com/jgm/pandoc-types/blob/1cf21a602535b6b263fef9548521353912115d87/src/Text/Pandoc/Definition.hs#L284) and [MultiMarkdown](https://fletcher.github.io/MultiMarkdown-6/syntax/raw.html).


**`@id`**: `stencila:RawBlock`

## Properties

The `RawBlock` type has these properties:

| Name                  | Aliases                                                                                                            | `@id`                                        | Type                                                                                                                      | Description                                                                             | Inherited from                                                                                   |
| --------------------- | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                  | -                                                                                                                  | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The identifier for this item.                                                           | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `format`              | -                                                                                                                  | `stencila:format`                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The format of the raw content.                                                          | -                                                                                                |
| `content`             | -                                                                                                                  | `stencila:content`                           | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                               | The raw content.                                                                        | -                                                                                                |
| `compilationDigest`   | `compilation-digest`, `compilation_digest`                                                                         | `stencila:compilationDigest`                 | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)    | A digest of the `format` and `content` properties.                                      | -                                                                                                |
| `compilationMessages` | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` | `stencila:compilationMessages`               | [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-message.md)* | Messages generated while parsing and transpiling the `content` into the `css` property. | -                                                                                                |
| `css`                 | -                                                                                                                  | `stencila:css`                               | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | A Cascading Style Sheet (CSS) generated from the `content`.                             | -                                                                                                |
| `authors`             | `author`                                                                                                           | [`schema:author`](https://schema.org/author) | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                         | The authors of the content.                                                             | -                                                                                                |
| `provenance`          | -                                                                                                                  | `stencila:provenance`                        | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)*      | A summary of the provenance of the content.                                             | -                                                                                                |

## Related

The `RawBlock` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `RawBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | ⚠️ High loss |            | 🔶 Beta              | Encoded using implemented function |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |                                    |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |                                    |

## Bindings

The `RawBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.org/RawBlock.jsonld)
- [JSON Schema](https://stencila.org/RawBlock.schema.json)
- Python class [`RawBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/raw_block.py)
- Rust struct [`RawBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/raw_block.rs)
- TypeScript class [`RawBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/RawBlock.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `RawBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                                                                                                          | Strategy                                    |
| --------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `format`  | Min+       | Fixed as Markdown                                                                                                                                    | `String::from("markdown")`                  |
|           | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                                        | `r"[a-zA-Z0-9]{1,10}"`                      |
|           | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary()`                       |
| `content` | Min+       | Generate a simple fixed string.                                                                                                                      | `Cord::from("content")`                     |
|           | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which <br><br>when leading or trailing causes issues for Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)` |
|           | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                                     | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`   |
|           | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary().prop_map(Cord::from)`  |

## Source

This documentation was generated from [`RawBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/RawBlock.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
