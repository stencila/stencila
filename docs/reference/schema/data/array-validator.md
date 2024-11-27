# Array Validator

**A validator specifying constraints on an array node.**

**`@id`**: `stencila:ArrayValidator`

## Properties

The `ArrayValidator` type has these properties:

| Name             | Aliases                              | `@id`                                | Type                                                                                                  | Description                                                                                 | Inherited from                                                                                   |
| ---------------- | ------------------------------------ | ------------------------------------ | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`             | -                                    | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)       | The identifier for this item.                                                               | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `itemsNullable`  | `items-nullable`, `items_nullable`   | `stencila:itemsNullable`             | [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)     | Whether items can have the value `Node::Null`                                               | -                                                                                                |
| `itemsValidator` | `items-validator`, `items_validator` | `stencila:itemsValidator`            | [`Validator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/validator.md) | Another validator node specifying the constraints on all items in the array.                | -                                                                                                |
| `contains`       | -                                    | `stencila:contains`                  | [`Validator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/validator.md) | An array node is valid if at least one of its items is valid against the `contains` schema. | -                                                                                                |
| `minItems`       | `min-items`, `min_items`             | `stencila:minItems`                  | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)     | An array node is valid if its size is greater than, or equal to, this value.                | -                                                                                                |
| `maxItems`       | `max-items`, `max_items`             | `stencila:maxItems`                  | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)     | An array node is valid if its size is less than, or equal to, this value.                   | -                                                                                                |
| `uniqueItems`    | `unique-items`, `unique_items`       | `stencila:uniqueItems`               | [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)     | A flag to indicate that each value in the array should be unique.                           | -                                                                                                |

## Related

The `ArrayValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `ArrayValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |            | 🔶 Beta              | Encoded using implemented function |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
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

The `ArrayValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ArrayValidator.jsonld)
- [JSON Schema](https://stencila.org/ArrayValidator.schema.json)
- Python class [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/array_validator.py)
- Rust struct [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/array_validator.rs)
- TypeScript class [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/ArrayValidator.ts)

## Source

This documentation was generated from [`ArrayValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/ArrayValidator.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
