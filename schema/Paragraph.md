# Paragraph

The `Paragraph` schema represents a paragraph, or a block of text. It can contain any valid [`InlineContent`](/schema/InlineContent) nodes.

## Examples

### Simple

```json validate
{
  "type": "Paragraph",
  "content": ["Some text content representing ideas expressed as words."]
}
```

### Nested Content

```json validate
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

## Related

### HTML

`Paragraph` is analagous to the HTML [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p) element.

### JATS

`Paragraph` is analagous to the JATS [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/p.html) element.

### mdast

`Paragraph` is analagous to the mdast [`Paragraph`](https://github.com/syntax-tree/mdast#Paragraph) node.

### OpenDocument

`Paragraph` is analagous to the OpenDocument
[`<text:p>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415138_253892949)
element.

### Pandoc

`Paragraph` is analagous to the Pandoc
[`Para`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L220)
type.
