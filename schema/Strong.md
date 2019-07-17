# Strong

The `Strong` schema represents strongly emphasised content. It can contain any valid [`InlineContent`](/schema/InlineContent) nodes.

## Examples

### Simple

```json validate
{
  "type": "Strong",
  "content": ["Some important information"]
}
```

### Nested types

```json validate
{
  "type": "Strong",
  "content": [
    "Some ",
    { "type": "Delete", "content": ["important"] },
    "essential information"
  ]
}
```

## Related

### JATS

`Strong` is analagous, and structurally similar to, the JATS [`<bold>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/aut-elem-sec-intro.html) type.

### mdast

`Strong` is analagous to the mdast [`Strong`](https://github.com/syntax-tree/mdast#strong) node type.

### OpenDocument

`Strong` is similar to the OpenDocument
[`<style:font-adornments>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1417910_253892949)
attribute.
