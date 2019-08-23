---
title: ThematicBreak
authors: []
---

include: ../public/ThematicBreak.schema.md
:::
A thematic break, such as a scene change in a story, a transition to another topic, or a new document.

| Entity | type | The name of the type and all descendant types. | string |
| ------ | ---- | ---------------------------------------------- | ------ |
| Entity | id   | The identifier for this item.                  | string |

:::

The `ThematicBreak` schema represents a thematic break, such as a scene change in a story, a transition to another topic, or a new document.

The way a thematic break is represented can vary from one format to another, in markdown and HTML for example it is often represented as a horizontal rule but in text editors can be represented as either a horizontal rule or a page break.

# Examples

A thematic break.

```json validate import=example
{
  "type": "ThematicBreak"
}
```

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## JATS

`ThematicBreak` does not have a similar counterpart in JATS. The [`<hr>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/hr.html) type is defined as an explicit horizontal rule and only recommended to be used in a table, whereas the Stencila `ThematicBreak` type can be used in broader contexts.

```jats export=example


```

## mdast

`ThematicBreak` is analogous to the mdast [`* * *`](https://github.com/syntax-tree/mdast#ThematicBreak) node type.

```markdown export=example
---
```

## OpenDocument

`ThematicBreak` is similar to the OpenDocument [`<text:soft-page-break>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#element-text_soft-page-break) element.

[//]: # 'WIP: JATS needs fixing'
