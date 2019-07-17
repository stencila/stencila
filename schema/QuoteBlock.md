# Quote Block

The `QuoteBlock` schema represents an extended quoted sections.

## Examples

### Simple

```json validate
{
  "type": "QuoteBlock",
  "content": [
    {
      "type": "Paragraph",
      "content": [
        "If you wish to make an apple pie from scratch, you must first invent the universe.",
        "by Carl Sagan - Cosmos"
      ]
    }
  ]
}
```

### With Attribution URI

```json validate
{
  "type": "QuoteBlock",
  "content": [
    {
      "type": "Paragraph",
      "content": [
        "If you wish to make an apple pie from scratch, you must first invent the universe.",
        "by Carl Sagan — Cosmos"
      ]
    }
  ],
  "citation": "https://www.goodreads.com/quotes/32952-if-you-wish-to-make-an-apple-pie-from-scratch"
}
```

## Related

### HTML

`QuoteBlock` is analagous to the HTML [`<blockquote>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote) element.

### JATS

`Quote` is analogous to the JATS
[`<disp-quote>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/disp-quote.html)
type, and the [`<attrib>` element](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/attrib.html) can be used for the `citation` field from the Stencila schema.

### mdast

`QuoteBlock` is analagous to the mdast [`Blockquote`](https://github.com/syntax-tree/mdast#blockquote) node type.

### OpenDocument

`QuoteBlock` does not have an analogous OpenDocument element.

### Pandoc

`QuoteBlock` is analagous to the Pandoc
[`BlockQuote`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L224)
type.
