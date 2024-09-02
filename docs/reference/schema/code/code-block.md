# Code Block

**A code block.**

**`@id`**: `stencila:CodeBlock`

## Properties

The `CodeBlock` type has these properties:

| Name                  | Aliases                                        | `@id`                                                                  | Type                                                                                                                 | Description                              | Inherited from                                                                                           |
| --------------------- | ---------------------------------------------- | ---------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `id`                  | -                                              | [`schema:id`](https://schema.org/id)                                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                      | The identifier for this item.            | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)         |
| `code`                | -                                              | `stencila:code`                                                        | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                          | The code.                                | [`CodeStatic`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-static.md) |
| `programmingLanguage` | `programming-language`, `programming_language` | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                      | The programming language of the code.    | [`CodeStatic`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-static.md) |
| `authors`             | `author`                                       | [`schema:author`](https://schema.org/author)                           | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                    | The authors of the code.                 | [`CodeStatic`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-static.md) |
| `provenance`          | -                                              | `stencila:provenance`                                                  | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)* | A summary of the provenance of the code. | [`CodeStatic`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-static.md) |

## Related

The `CodeBlock` type is related to these types:

- Parents: [`CodeStatic`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-static.md)
- Children: none

## Formats

The `CodeBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes                                                                                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | -------------------------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |                                                                                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🟢 No loss    |           | 🚧 Under development | Encoded as [`<pre>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre)                |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🟢 No loss    |           | 🚧 Under development | Encoded as [`<code>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/code.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | 🟢 No loss    | 🟢 No loss | ⚠️ Alpha            | Encoded using implemented function                                                                 |
| [MyST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)                | 🟢 No loss    | 🟢 No loss | ⚠️ Alpha            |                                                                                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |                                                                                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                                                                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |                                                                                                    |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |           | 🚧 Under development |                                                                                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |                                                                                                    |

## Bindings

The `CodeBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.org/CodeBlock.jsonld)
- [JSON Schema](https://stencila.org/CodeBlock.schema.json)
- Python class [`CodeBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_block.py)
- Rust struct [`CodeBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_block.rs)
- TypeScript class [`CodeBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeBlock.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `CodeBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property              | Complexity | Description                                                                                                                    | Strategy                                      |
| --------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------- |
| `code`                | Min+       | Generate a simple fixed string of code.                                                                                        | `Cord::from("code")`                          |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which<br><br>can be problematic in Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`   |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                               | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`     |
|                       | Max        | Generate an arbitrary string.                                                                                                  | `String::arbitrary().prop_map(Cord::from)`    |
| `programmingLanguage` | Min+       | Do not generate a programming language.                                                                                        | `None`                                        |
|                       | Low+       | Generate one of the well known programming language short names.                                                               | `option::of(r"(cpp)\|(js)\|(py)\|(r)\|(ts)")` |
|                       | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                  | `option::of(r"[a-zA-Z0-9]{1,10}")`            |
|                       | Max        | Generate an arbitrary string.                                                                                                  | `option::of(String::arbitrary())`             |

## Source

This documentation was generated from [`CodeBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeBlock.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
