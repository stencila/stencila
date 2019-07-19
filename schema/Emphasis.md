---
title: Emphasis
authors: []
---

include: ../built/Emphasis.schema.md
:::
Emphasised content.

| Thing | type           | The name of the type and all descendant types. | string |
| ----- | -------------- | ---------------------------------------------- | ------ |
| Thing | id             | The identifier for this item.                  | string |
| Thing | alternateNames | Alternate names (aliases) for the item.        | array  |
| Thing | description    | A description of the item.                     | string |
| Thing | meta           | Metadata associated with this item.            | object |
| Thing | name           | The name of the item.                          | string |
| Thing | url            | The URL of the item.                           | string |
| Mark  | content        | The content that is marked.                    |        |
| array |                |                                                |        |

:::

# Formats

## JSON

To illustrate how `Emphasis` nodes are represented in alternative formats, we'll use the following example, in context, within a `Paragraph`:

```json import=inpara
{
  "type": "Paragraph",
  "content": [
    "The following content has extra ",
    {
      "type": "Emphasis",
      "content": ["emphasis"]
    }
  ]
}
```

## HTML

HTML natively supports `Emphasis` nodes with the [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em) element.

```html export=inpara
<p>The following content has extra <em>emphasis</em></p>
```

## JATS

The JATS equivalent to `Emphasis` is the [`<italic>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/italic.html) element.

```jats export=inpara
<p>The following content has extra <italic>emphasis</italic></p>

```

## YAML

```yaml export=inpara
type: Paragraph
content:
  - 'The following content has extra '
  - type: Emphasis
    content:
      - emphasis
```

## Markdown

Emphasis in Markdown can be achieved with underscores (`_`) or asterisks (`*`). See also the [MDAST reference](https://github.com/syntax-tree/mdast#emphasis).

```md export=inpara
The following content has extra _emphasis_
```

## Pandoc

The equivalent of `Emphasis` in Pandoc is the [`Emph`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L256) element. The above example in Pandoc JSON:

```pandoc export=inpara format=pandoc
{
"blocks": [
{
"t": "Para",
"c": [
{
"t": "Str",
"c": "The following content has extra "
},
{
"t": "Emph",
"c": [
{
"t": "Str",
"c": "emphasis"
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
