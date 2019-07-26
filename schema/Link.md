---
title: Link
authors: []
---

# Link

The `Link` schema represents a hyperlink to other pages, sections within the same document, resources, or any URL.

include: ../built/Link.schema.md
:::
A link.

| Thing | type           | The name of the type and all descendant types.         | string |
| ----- | -------------- | ------------------------------------------------------ | ------ |
| Thing | id             | The identifier for this item.                          | string |
| Thing | alternateNames | Alternate names (aliases) for the item.                | array  |
| Thing | description    | A description of the item.                             | string |
| Thing | meta           | Metadata associated with this item.                    | object |
| Thing | name           | The name of the item.                                  | string |
| Thing | url            | The URL of the item.                                   | string |
| Link  | content        |                                                        | array  |
| Link  | target         |                                                        | string |
| Link  | relation       | The relation between the target and the current thing. | string |

:::

# Examples

This is a simple `Link` that will be encoded to different formats as an example.

```json import=ex1
{
  "type": "Link",
  "content": ["Stencila’s website"],
  "target": "https://stenci.la"
}
```

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## HTML

`Link` is analogous to the HTML [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a) element.

```html export=ex1
<a href="https://stenci.la">Stencila’s website</a>
```

## JATS

`Link` is analogous to the JATS [`<ext-link>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/ext-link.html) element.

```jats export=ex1
<p><ext-link ext-link-type="uri" xlink:href="https://stenci.la">Stencila’s website</ext-link></p>

```

## Markdown

`Link` is analogous to the mdast [`Link`](https://github.com/syntax-tree/mdast#link) node.

```md export=ex1
[Stencila’s website](https://stenci.la)
```

## OpenDocument

`Link` is analogous to the OpenDocument [`<text:a>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415212_253892949) element. This [`odt`](link-ex1.out.odt){export=ex1} file was generated from the above example.

## Pandoc

`Link` is analogous to the Pandoc [`Link`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L270) type.

```pandoc export=ex1
{
"blocks": [
{
"t": "Para",
"c": [
{
"t": "Link",
"c": [
[
"",
[],
[]
],
[
{
"t": "Str",
"c": "Stencila’s website"
}
],
[
"https://stenci.la",
""
]
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
