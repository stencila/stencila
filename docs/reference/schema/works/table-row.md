# Table Row

**A row within a Table.**

**`@id`**: `stencila:TableRow`

## Properties

The `TableRow` type has these properties:

| Name      | Aliases                | `@id`                                | Type                                                                                                           | Description                   | Inherited from                                                                                   |
| --------- | ---------------------- | ------------------------------------ | -------------------------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`      | -                      | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                | The identifier for this item. | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `cells`   | `cell`                 | `stencila:cells`                     | [`TableCell`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table-cell.md)*       | An array of cells in the row. | -                                                                                                |
| `rowType` | `row-type`, `row_type` | `stencila:rowType`                   | [`TableRowType`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table-row-type.md) | The type of row.              | -                                                                                                |

## Related

The `TableRow` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `TableRow` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding      | Status                 | Notes                                                                             |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------- | ---------------------- | --------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |               | 🚧 Under development    | Encoded as [`<tr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |                  |               | 🚧 Under development    |                                                                                   |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | 🔷 Low loss       | 🔷 Low loss    | ⚠️ Alpha               |                                                                                   |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |               | ⚠️ Alpha               |                                                                                   |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                   |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                   |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss     | 🔶 Beta                 |                                                                                   |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                   |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                   |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss     | 🟢 Stable               |                                                                                   |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |               | 🟢 Stable               |                                                                                   |

## Bindings

The `TableRow` type is represented in these bindings:

- [JSON-LD](https://stencila.org/TableRow.jsonld)
- [JSON Schema](https://stencila.org/TableRow.schema.json)
- Python class [`TableRow`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/table_row.py)
- Rust struct [`TableRow`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_row.rs)
- TypeScript class [`TableRow`](https://github.com/stencila/stencila/blob/main/ts/src/types/TableRow.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `TableRow` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                   | Strategy                                         |
| -------- | ---------- | --------------------------------------------- | ------------------------------------------------ |
| `cells`  | Min+       | Generate a single, arbitrary, table cell.     | `vec(TableCell::arbitrary(), size_range(1..=1))` |
|          | Low+       | Generate two, arbitrary, table cells.         | `vec(TableCell::arbitrary(), size_range(2..=2))` |
|          | High+      | Generate four, arbitrary, table cells.        | `vec(TableCell::arbitrary(), size_range(4..=4))` |
|          | Max        | Generate up to eight, arbitrary, table cells. | `vec(TableCell::arbitrary(), size_range(1..=8))` |

## Source

This documentation was generated from [`TableRow.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableRow.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.