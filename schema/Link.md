# Link

The `Link` schema represents a hyperlink to other pages, sections within the same document, resources, or any URL.

## Examples

### Simple

```json validate
{
  "type": "Link",
  "content": ["Stencila’s website"],
  "target": "https://stenci.la"
}
```

## Related

### HTML

`Link` is analagous to the HTML [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a) element.

### JATS

`Link` is analagous to the JATS [`<ext-link>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/ext-link.html) element.

### mdast

`Link` is analagous to the mdast [`Link`](https://github.com/syntax-tree/mdast#link) node.

### OpenDocument

`Link` is analagous to the OpenDocument
[`<text:a>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415212_253892949)
element.

### Pandoc

`Link` is analagous to the Pandoc
[`Link`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L270)
type.
