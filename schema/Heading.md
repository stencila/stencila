---
title: Heading
authors: []
---

include: ../built/Heading.schema.md
:::
Heading

| Thing   | type           | The name of the type and all descendant types. | string |
| ------- | -------------- | ---------------------------------------------- | ------ |
| Thing   | id             | The identifier for this item.                  | string |
| Thing   | alternateNames | Alternate names (aliases) for the item.        | array  |
| Thing   | description    | A description of the item.                     | string |
| Thing   | meta           | Metadata associated with this item.            | object |
| Thing   | name           | The name of the item.                          | string |
| Thing   | url            | The URL of the item.                           | string |
| Heading | depth          |                                                | number |
| Heading | content        | Content of the heading.                        | array  |

:::

# Examples

To illustrate how `Heading` nodes are represented in alternative formats, we'll use the following example.

```json import=heading
{
  "type": "Heading",
  "depth": 2,
  "content": ["Secondary Heading"]
}
```

For compatibility with HTML, only integer depths in the range 1–6 are supported.

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## HTML

HTML supports `Heading` nodes with the [`<h1>` to `<h6>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1) elements.

```html export=heading
<h2 id="secondary-heading">Secondary Heading</h2>
```

## JATS

JATS lacks a depth attribute so this is lost on conversion, the [`<title>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/title.html) element is used.

```jats export=heading
<sec>
  <title>Secondary Heading</title>
</sec>

```

## YAML

```yaml export=heading
type: Heading
depth: 2
content:
  - Secondary Heading
```

## Markdown

Markdown headings are denoted by a number of hashes (`#`) equal to the depth.

```markdown export=heading
## Secondary Heading
```

## Pandoc

The equivalent of `Heading` in Pandoc is the [`Header`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L233) element. The above example in Pandoc JSON:

```pandoc export=heading
{
"blocks": [
{
"t": "Header",
"c": [
2,
[
"",
[],
[]
],
[
{
"t": "Str",
"c": "Secondary Heading"
}
]
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
