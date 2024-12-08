# Table Cell

**A cell within a `Table`.**

**`@id`**: `stencila:TableCell`

## Properties

The `TableCell` type has these properties:

| Name         | Aliases                      | `@id`                                    | Type                                                                                                             | Description                        | Inherited from                                                                                   |
| ------------ | ---------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | ---------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`         | -                            | [`schema:id`](https://schema.org/id)     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                  | The identifier for this item.      | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `cellType`   | `cell-type`, `cell_type`     | `stencila:cellType`                      | [`TableCellType`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table-cell-type.md) | The type of cell.                  | -                                                                                                |
| `name`       | -                            | [`schema:name`](https://schema.org/name) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                  | The name of the cell.              | -                                                                                                |
| `columnSpan` | `column-span`, `column_span` | `stencila:colspan`                       | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                | How many columns the cell extends. | -                                                                                                |
| `rowSpan`    | `row-span`, `row_span`       | `stencila:rowspan`                       | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                | How many columns the cell extends. | -                                                                                                |
| `content`    | -                            | `stencila:content`                       | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                  | Contents of the table cell.        | -                                                                                                |

## Related

The `TableCell` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `TableCell` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                                                                             |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | --------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |                                                                                   |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development | Encoded as [`<td>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |                                                                                   |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              |                                                                                   |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              |                                                                                   |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              |                                                                                   |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              |                                                                                   |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | 🔷 Low loss   | 🔷 Low loss | 🔶 Beta              |                                                                                   |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                   |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |                                                                                   |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |                                                                                   |
| [IPYNB](https://github.com/stencila/stencila/blob/main/docs/reference/formats/ipynb.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                   |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                   |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                   |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                   |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                   |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                   |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                                                                   |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                   |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                   |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                                                                   |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                                                                   |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |                                                                                   |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |                                                                                   |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |                                                                                   |

## Bindings

The `TableCell` type is represented in these bindings:

- [JSON-LD](https://stencila.org/TableCell.jsonld)
- [JSON Schema](https://stencila.org/TableCell.schema.json)
- Python class [`TableCell`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/table_cell.py)
- Rust struct [`TableCell`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_cell.rs)
- TypeScript class [`TableCell`](https://github.com/stencila/stencila/blob/main/ts/src/types/TableCell.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `TableCell` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                             | Strategy                |
| --------- | ---------- | --------------------------------------- | ----------------------- |
| `content` | Min+       | Generate a single, arbitrary, paragraph | `table_cell_content(1)` |

## Source

This documentation was generated from [`TableCell.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableCell.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
