# Table Row

A `TableRow` type is primarily a container for a list of [`Table Cell`](/schema/TableCell) types.

## Examples

### Simple

This is the most basic form of `TableRow` you can have, only requiring the `content` field.

```json
{
  "type": "TableRow",
  "cells": [
    {
      "type": "TableCell",
      "content": [1]
    }
  ]
}
```

### An Empty Row

A `TableRow` can be empty and still valid, representing a table row with no
data. The `content` field however, is still required.

```json
{
  "type": "TableRow",
  "content": []
}
```

## Related

### JATS

`TableRow` is analagous to the JATS
[`<tr>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/tr.html) type.

### mdast

`TableRow` is analagous to the mdast [`TableRow`](https://github.com/syntax-tree/mdast#tablerow) node type.

### OpenDocument

`Table` is analagous to the OpenDocument
[`<table:table-row>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415588_253892949)
element.
