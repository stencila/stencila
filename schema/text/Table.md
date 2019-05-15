# Table

A `Table` type allows you to represent two-dimensional data, in a way
that can easily be translated into an XHTML table if needed. It is primarily a
container for [`Table Row`](/schema/TableRow) and [`Table Cell`](/schema/TableCell) types.

## Examples

### Simple

This is the most basic form of `Table` you can have. It contains no rows or
columns, and represents an empty table.

```json
{
  "type": "Table",
  "rows": []
}
```

### Table with Rows & Cells

The `Table` type can contain `rows`, which in turn contain `cells`.

```json
{
  "type": "Table",
  "rows": [
    {
      "type": "TableRow",
      "cells": [
        {
          "type": "TableCell",
          "position": [0, 0],
          "content": [1]
        },
        {
          "type": "TableCell",
          "position": [0, 1],
          "content": [2]
        }
      ]
    },
    {
      "type": "TableRow",
      "cells": [
        {
          "type": "TableCell",
          "position": [1, 0],
          "content": [1]
        },
        {
          "type": "TableCell",
          "position": [1, 1],
          "content": [2]
        }
      ]
    }
  ]
}
```

## Related

### JATS

`Table` is analagous, and structurally similar to, the JATS
[`<table>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/table.html) type.
They both store data in a way that is "intended to be converted easily to the
XHMTL table element."

### mdast

`Table` is analagous to the mdast
[`Table`](https://github.com/syntax-tree/mdast#table) node type, however note
that unlike the mdast equivalent, the Stencila variant does not contain the
stylistic `align` field.

### OpenDocument

`Table` is analagous to the OpenDocument
[`<table:table>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415586_253892949)
element, which is the root element for a table.
