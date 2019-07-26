---
title: Quote Block
authors: []
---

include: ../built/QuoteBlock.schema.md
:::
A section quoted from somewhere else.

| Entity     | type     | The name of the type and all descendant types. | string |
| ---------- | -------- | ---------------------------------------------- | ------ |
| Entity     | id       | The identifier for this item.                  | string |
| QuoteBlock | citation | The source of the quote                        | string |
| QuoteBlock | content  |                                                | array  |

:::

The `QuoteBlock` schema represents an extended quoted sections.

# Examples

## Simple

```json
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

## With Attribution URI

```json
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

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## HTML

`QuoteBlock` is analogous to the HTML [`<blockquote>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote) element.

### Simple

```html

```

### With Attribution URI

```html

```

## JATS

`Quote` is analogous to the JATS [`<disp-quote>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/disp-quote.html) type, and the [`<attrib>` element](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/attrib.html) can be used for the `citation` field from the Stencila schema.

### Simple

```jats

```

### With Attribution URI

```jats

```

## mdast

`QuoteBlock` is analogous to the mdast [`Blockquote`](https://github.com/syntax-tree/mdast#blockquote) node type.

### Simple

```markdown
```

### With Attribution URI

```markdown
```

## OpenDocument

`QuoteBlock` does not have an analogous OpenDocument element.

## Pandoc

`QuoteBlock` is analogous to the Pandoc [`BlockQuote`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L224) type.

### Simple

```pandoc
```

### With Attribution URI

```pandoc
```

[//]: # 'WIP: Needs QuoteBlock JSON Fixes'
