# Table Cell

A `TableCell` type represents a single cell in a [`Table`](/schema/Table). It contains properties
defining the contents, and optionally the dimensions of the cell within the table.

## Examples

### Simple

This is the most basic form of `TableCell` you can have, only requiring the `content` field.

```json
{
  "type": "TableCell",
  "content": [1]
}
```

### Table Cell with all properties defined

```json
{
  "type": "TableCell",
  "colspan": 2,
  "content": [1],
  "name": "myCustomLabel",
  "rowspan": 1
}
```

### Table Cell with Inline Content

The `contents` of a `TableCell` can contain values besides simple primitives like strings and numbers.
It can store any valid [`InlineContent`](/schema/InlineContent) such as emphasized text or even images.

```json
{
  "type": "TableCell",
  "content": [
    {
      "type": "Emphasis",
      "content": ["Some emphasized content"]
    }
  ]
}
```

## Related

### JATS

`TableCell` is analagous to the JATS
[`<td>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/td.html) type.
While the JATS `<td>` element must be contained "in the body of a table, as
opposed to a cell in the table header", the Stencila equivalent does not
impose such restrictions.

### mdast

`TableCell` is analagous to the mdast [`TableCell`](https://github.com/syntax-tree/mdast#tablecell) node type.

### OpenDocument

`TableCell` is analagous to the OpenDocument
[`<table:table-cell>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415590_253892949)
element, which:

> can contain paragraphs and other text content as well as sub tables. Table cells may span multiple columns and rows. Table cells may be empty.
