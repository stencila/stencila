---
title: Paragraph
authors: []
---

include: ../built/Paragraph.schema.md
:::
Paragraph

| Entity    | type    | The name of the type and all descendant types. | string |
| --------- | ------- | ---------------------------------------------- | ------ |
| Entity    | id      | The identifier for this item.                  | string |
| Paragraph | content |                                                | array  |

:::

The `Paragraph` schema represents a paragraph, or a block of text. It can contain any valid [`InlineContent`](/schema/InlineContent) nodes.

# Examples

## Simple

```json validate import=simple
{
  "type": "Paragraph",
  "content": ["Some text content representing ideas expressed as words."]
}
```

## Nested Content

```json validate import=nested
{
  "type": "Paragraph",
  "content": [
    "Some text with some",
    {
      "type": "Emphasis",
      "content": ["emphasised words"]
    },
    " and ",
    {
      "type": "Strong",
      "content": ["some strongly emphasised words"]
    }
  ]
}
```

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## HTML

`Paragraph` is analogous to the HTML [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p) element.

### Simple

```html export=simple
<p>Some text content representing ideas expressed as words.</p>
```

### Nested Content

```html export=nested
<p>
  Some text with some<em>emphasised words</em> and
  <strong>some strongly emphasised words</strong>
</p>
```

## JATS

`Paragraph` is analogous to the JATS [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/p.html) element.

### Simple

```jats export=simple
<p>Some text content representing ideas expressed as words.</p>

```

### Nested Content

```jats export=nested
<p>Some text with some<italic>emphasised words</italic> and <bold>some strongly emphasised words</bold></p>

```

## mdast

`Paragraph` is analogous to the mdast [`Paragraph`](https://github.com/syntax-tree/mdast#Paragraph) node.

### Simple

```markdown export=simple
Some text content representing ideas expressed as words.
```

### Nested Content

```markdown export=nested
Some text with some*emphasised words* and **some strongly emphasised words**
```

## OpenDocument

`Paragraph` is analogous to the OpenDocument [`<text:p>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415138_253892949) element.

## Pandoc

`Paragraph` is analogous to the Pandoc [`Para`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L220) type.

### Simple

```pandoc export=simple
{
"blocks": [
{
"t": "Para",
"c": [
{
"t": "Str",
"c": "Some text content representing ideas expressed as words."
}
]
}
],
"pandoc-api-version": [
1,
17,
5,
4
],
"meta": {}
}
```

### Nested Content

```pandoc export=nested
{
"blocks": [
{
"t": "Para",
"c": [
{
"t": "Str",
"c": "Some text with some"
},
{
"t": "Emph",
"c": [
{
"t": "Str",
"c": "emphasised words"
}
]
},
{
"t": "Str",
"c": " and "
},
{
"t": "Strong",
"c": [
{
"t": "Str",
"c": "some strongly emphasised words"
}
]
}
]
}
],
"pandoc-api-version": [
1,
17,
5,
4
],
"meta": {}
}
```
