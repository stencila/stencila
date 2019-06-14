# List

The `List` schema represents a collection of items, which can be ordered or unordered.

## Examples

### Simple

If an `order` field is not defined, the list is assumed to be `unordered`.

```json
{
  "type": "List",
  "items": ["Item One", "Item Two", "Item Three"]
}
```

### Nested Ordered List Inside an Unordered List

A [`ListItem`](/schema/ListItem) can contain any valid [`BlockContent`](/schema/BlockContent), meaning that lists can be nested and/or contain other block elements.

```json
{
  "type": "List",
  "items": [
    "Item One",
    {
      "type": "ListItem",
      "content": [
        "This is a nested item",
        {
          "type": "List",
          "order": "ordered",
          "items": ["Nested Item One", "Nested Item Two", "Nested Item Three"]
        }
      ]
    },
    "Item Three"
  ]
}
```

### Checklist

```json
{
  "type": "List",
  "items": [
    {
      "type": "ListItem",
      "checked": false,
      "content": ["Todo item"]
    },
    {
      "type": "ListItem",
      "checked": true,
      "content": ["Completed todo item"]
    }
  ]
}
```

## Related

### JATS

`List` is analagous, and structurally similar to, the JATS
[`<list>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/list.html)
type. Note that JATS type requires the `List` to contain at least one list
item, but the Stencila equivalent can be empty.

### mdast

`List` is analagous to the mdast
[`List`](https://github.com/syntax-tree/mdast#list) node type.

### OpenDocument

`List` is analagous to the OpenDocument
[`<text:list>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415148_253892949)
element.
