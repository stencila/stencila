# Table Cell

**A cell within a `Table`.**

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name     | `@id`                                                         | Type                                                                                     | Description                                                           | Inherited from            |
| -------- | ------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | --------------------------------------------------------------------- | ------------------------- |
| cellType | [stencila:cellType](https://schema.stenci.la/cellType.jsonld) | 'Data', 'Header'                                                                         | Indicates whether the cell is a header or data. See note [1](#notes). | [TableCell](TableCell.md) |
| colspan  | [stencila:colspan](https://schema.stenci.la/colspan.jsonld)   | integer                                                                                  | How many columns the cell extends. See note [2](#notes).              | [TableCell](TableCell.md) |
| content  | [stencila:content](https://schema.stenci.la/content.jsonld)   | Array of [BlockContent](BlockContent.md) _or_ Array of [InlineContent](InlineContent.md) | Contents of the table cell.                                           | [TableCell](TableCell.md) |
| id       | [schema:id](https://schema.org/id)                            | string                                                                                   | The identifier for this item.                                         | [Entity](Entity.md)       |
| meta     | [stencila:meta](https://schema.stenci.la/meta.jsonld)         | object                                                                                   | Metadata associated with this item.                                   | [Entity](Entity.md)       |
| name     | [schema:name](https://schema.org/name)                        | string                                                                                   | The name of the cell. See note [3](#notes).                           | [TableCell](TableCell.md) |
| rowspan  | [stencila:rowspan](https://schema.stenci.la/rowspan.jsonld)   | integer                                                                                  | How many columns the cell extends. See note [4](#notes).              | [TableCell](TableCell.md) |

## Notes

1. **cellType** : When `header`, the cell is similar to the HTML [`<th>` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th)). When `data`, the cell is similar to the HTML [`<td>` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td)).
2. **colspan** : Based on the HTML `colspan` attribute for [table cells](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td).
3. **name** : Cell's have an implicit name derived from their position in the table e.g. `C4` for the cell in the third column and fourth row. However this name can be overridden with an explicit name, e.g. `rate`.
4. **rowspan** : Based on the HTML `rowspan` attribute for [table cells](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td).

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/TableCell.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/TableCell.schema.json)
- Python [`class TableCell`](https://stencila.github.io/schema/python/docs/types.html#schema.types.TableCell)
- TypeScript [`interface TableCell`](https://stencila.github.io/schema/ts/docs/interfaces/tablecell.html)
- R [`class TableCell`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct TableCell`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.TableCell.html)

## Source

This documentation was generated from [TableCell.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/TableCell.schema.yaml).
