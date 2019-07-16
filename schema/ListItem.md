# List Item

The `ListItem` schema represents a collection of items, which can be ordered or unordered.

## Examples

### Simple

```json
{
  "type": "ListItem",
  "content": ["List Item Content"]
}
```

### Nested Ordered List Inside an Unordered List

A list item can contain any valid `Node`, meaning that lists can be nested and/or contain other block elements.

```json
{
  "type": "ListItem",
  "content": [
    "List Item Content",
    {
      "type": "List",
      "order": "ordered",
      "items": ["Nested Item One"]
    }
  ]
}
}
```

### Checklist

If the `checked` field is present, the `ListItem` is considered to be
completable (either done or not done). To indicate that a `ListItem` is _not_
completable, omit the `checked` field.

```json
{
  "type": "ListItem",
  "checked": true,
  "content": ["Completed todo item"]
}
```

## Related

### JATS

`ListItem` is analagous to the JATS
[`<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/list-item.html)
type.
Note that JATS only permits the `ListItem` to contain either a
[`Paragraph`](/schema/Paragraph) element, or another [`List`](/schema/List), while the Stencila equivalent is closer to HTML and accepts any valid `Node`.

### mdast

`ListItem` is analagous to the mdast
[`ListItem`](https://github.com/syntax-tree/mdast#listitem) node type.

### OpenDocument

`ListItem` is analagous to the OpenDocument
[`<text:list-item>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415154_253892949)
element. Note that OpenDocument only permits the `ListItem` to contain either
a [`Paragraph`](/schema/Paragraph) element, or another
[`List`](/schema/List), while the Stencila equivalent is closer to HTML and
accepts any valid `Node`.
