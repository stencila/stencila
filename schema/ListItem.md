---
title: List Item
authors: []
---

include: ../built/ListItem.schema.md
:::
A single item in a list.

| Entity   | type    | The name of the type and all descendant types. | string  |
| -------- | ------- | ---------------------------------------------- | ------- |
| Entity   | id      | The identifier for this item.                  | string  |
| ListItem | content |                                                | array   |
| ListItem | checked |                                                | boolean |

:::

# Examples

## Simple Item

```json validate import=simple
{
  "type": "ListItem",
  "content": ["List Item Content"]
}
```

## Nested Ordered List Inside an Unordered List

A list item can contain any valid `Node`, meaning that lists can be nested and/or contain other block elements.

```json validate import=nested
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
```

## Checklist

If the `checked` field is present, the `ListItem` is considered to be completable (either done or not done). To indicate that a `ListItem` is _not_ completable, omit the `checked` field.

```json validate import=checklist
{
  "type": "ListItem",
  "checked": true,
  "content": ["Completed todo item"]
}
```

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## JATS

`ListItem` is analogous to the JATS [`<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/list-item.html) type. Note that JATS only permits the `ListItem` to contain either a [`Paragraph`](/schema/Paragraph) element, or another [`List`](/schema/List), while the Stencila equivalent is closer to HTML and accepts any valid `Node`.

### Simple Item

```jats

```

### Nested Ordered List Inside an Unordered List

```jats

```

### Checklist

```jats

```

## mdast

`ListItem` is analogous to the mdast [`ListItem`](https://github.com/syntax-tree/mdast#listitem) node type.

### Simple Item

```markdown export=simple
- [ ]
```

### Nested Ordered List Inside an Unordered List

```markdown export=nested
- [ ]
```

### Checklist

```markdown export=checklist
- [x]
```

## OpenDocument

`ListItem` is analogous to the OpenDocument [`<text:list-item>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415154_253892949) element. Note that OpenDocument only permits the `ListItem` to contain either a [`Paragraph`](/schema/Paragraph) element, or another [`List`](/schema/List), while the Stencila equivalent is closer to HTML and accepts any valid `Node`.

[//]: # 'WIP: Needs JATS and markdown Fixes'
