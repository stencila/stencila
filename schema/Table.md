---
title: Table
authors: []
---

include: ../public/Table.schema.md
:::
A table.

| Entity       | type           | The name of the type and all descendant types.                                | string |
| ------------ | -------------- | ----------------------------------------------------------------------------- | ------ |
| Entity       | id             | The identifier for this item.                                                 | string |
| Thing        | alternateNames | Alternate names (aliases) for the item.                                       | array  |
| Thing        | description    | A description of the item.                                                    | string |
| Thing        | meta           | Metadata associated with this item.                                           | object |
| Thing        | name           | The name of the item.                                                         | string |
| Thing        | url            | The URL of the item.                                                          | string |
| CreativeWork | authors        | The authors of this this creative work.                                       | array  |
| CreativeWork | citations      | Citations or references to other creative works, such as another publication, |        |

web page, scholarly article, etc. | array | | CreativeWork | content | The structured content of this creative work c.f. property \`text\`. | array | | CreativeWork | dateCreated | Date/time of creation. | | | CreativeWork | dateModified | Date/time of most recent modification. | | | CreativeWork | datePublished | Date of first publication. | | | CreativeWork | editors | Persons who edited the CreativeWork. | array | | CreativeWork | funders | Person or organisation that funded the CreativeWork. | array | | CreativeWork | isPartOf | An item or other CreativeWork that this CreativeWork is a part of. | | | CreativeWork | licenses | License documents that applies to this content, typically indicated by URL. | array | | CreativeWork | parts | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | array | | CreativeWork | publisher | A publisher of the CreativeWork. | | | CreativeWork | text | The textual content of this creative work. | string | | CreativeWork | title | | string | | CreativeWork | version | | | | Table | rows | Rows of cells in the table. | array |
:::

A `Table` type allows you to represent two-dimensional data, in a way that can easily be translated into an XHTML table if needed. It is primarily a container for [`Table Row`](/schema/TableRow) and [`Table Cell`](/schema/TableCell) types.

# Examples

## Simple

This is the most basic form of `Table` you can have. It contains no rows or columns, and represents an empty table.

```json import=simple
{
  "type": "Table",
  "rows": []
}
```

## Table with Rows & Cells

The `Table` type can contain `rows`, which in turn contain `cells`.

```json import=complex
{
  "type": "Table",
  "rows": [
    {
      "type": "TableRow",
      "cells": [
        {
          "type": "TableCell",
          "content": ["one"]
        },
        {
          "type": "TableCell",
          "content": ["two"]
        }
      ]
    },
    {
      "type": "TableRow",
      "cells": [
        {
          "type": "TableCell",
          "content": ["three"]
        },
        {
          "type": "TableCell",
          "content": ["four"]
        }
      ]
    }
  ]
}
```

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## JATS

`Table` is analogous, and structurally similar to, the JATS [`<table>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/table.html) type. They both store data in a way that is "intended to be converted easily to the XHMTL table element."

### Simple

```jats export=simple
<table>
  <tbody>
  </tbody>
</table>

```

### Table with Rows & Cells

```jats export=complex
<table>
  <thead>
    <tr>
      <th><p>one</p></th>
      <th><p>two</p></th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><p>three</p></td>
      <td><p>four</p></td>
    </tr>
  </tbody>
</table>

```

## mdast

`Table` is analogous to the mdast [`Table`](https://github.com/syntax-tree/mdast#table) node type, however note that unlike the mdast equivalent, the Stencila variant does not contain the stylistic `align` field.

### Simple

```markdown export=simple
| |
```

### Table with Rows & Cells

```markdown export=complex
| one   | two  |
| ----- | ---- |
| three | four |
```

## OpenDocument

`Table` is analogous to the OpenDocument [`<table:table>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415586_253892949) element, which is the root element for a table.
