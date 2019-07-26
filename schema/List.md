---
title: List
authors: []
---

include: ../built/List.schema.md
:::
A list of items.

| Entity | type  | The name of the type and all descendant types. | string |
| ------ | ----- | ---------------------------------------------- | ------ |
| Entity | id    | The identifier for this item.                  | string |
| List   | items | The items in the list                          | array  |
| List   | order | Type of ordering.                              |        |

:::

The `List` schema represents a collection of items, which can be ordered or unordered.

# Examples

## Unordered

If an `order` field is not defined, the list is assumed to be `unordered`.

```json import=unordered
{
  "type": "List",
  "items": [
    { "type": "ListItem", "content": ["Item One"] },
    { "type": "ListItem", "content": ["Item Two"] },
    { "type": "ListItem", "content": ["Item Three"] }
  ]
}
```

## Nested Ordered List Inside an Unordered List

A [`ListItem`](/schema/ListItem) can contain any valid `Node`, meaning that lists can be nested and/or contain other block elements.

```json import=nested
{
  "type": "List",
  "items": [
    { "type": "ListItem", "content": "Item One" },
    {
      "type": "ListItem",
      "content": [
        "This is a nested item",
        {
          "type": "List",
          "order": "ordered",
          "items": [
            { "type": "ListItem", "content": ["Nested Item One"] },
            { "type": "ListItem", "content": ["Nested Item Two"] },
            { "type": "ListItem", "content": ["Nested Item Three"] }
          ]
        }
      ]
    },
    { "type": "ListItem", "content": ["Item Three"] }
  ]
}
```

## Checklist

```json import=checklist
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

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## JSON

## JATS

`List` is analogous, and structurally similar to, the JATS [`<list>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/list.html) type. Note that JATS type requires the `List` to contain at least one list item, but the Stencila equivalent can be empty.

### Unordered

```jats export=unordered
<list list-type="bullet">
  <list-item>
    <p>Item One</p>
  </list-item>
  <list-item>
    <p>Item Two</p>
  </list-item>
  <list-item>
    <p>Item Three</p>
  </list-item>
</list>

```

### Nested Ordered List Inside an Unordered List

```jats export=nested
<list list-type="bullet">
  <list-item>
    <p>Item One</p>
  </list-item>
  <list-item>
    <p>This is a nested item</p>
    <list list-type="bullet">
      <list-item>
        <p>Nested Item One</p>
      </list-item>
      <list-item>
        <p>Nested Item Two</p>
      </list-item>
      <list-item>
        <p>Nested Item Three</p>
      </list-item>
    </list>
  </list-item>
  <list-item>
    <p>Item Three</p>
  </list-item>
</list>

```

### Checklist

```jats export=checklist
<list list-type="bullet">
  <list-item>
    <p>Todo item</p>
  </list-item>
  <list-item>
    <p>Completed todo item</p>
  </list-item>
</list>

```

## mdast

`List` is analogous to the mdast [`List`](https://github.com/syntax-tree/mdast#list) node type.

### Unordered

```markdown export=unordered
- [ ]
- [ ]
- [ ]
```

### Nested Ordered List Inside an Unordered List

```markdown export=nested
- [ ]
- [ ] ## -
  -
- [ ]
```

### Checklist

```markdown export=checklist
- [ ]
- [x]
```

## OpenDocument

`List` is analogous to the OpenDocument [`<text:list>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415148_253892949) element.

[//]: # 'WIP: Needs markdown Fixes'
