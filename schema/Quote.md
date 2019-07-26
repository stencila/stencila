---
title: Quote
authors: []
---

include: ../built/Quote.schema.md
:::
Inline, quoted content. Analagous to, - HTML \[\`&lt;q>\` element](https&#x3A;//developer.mozilla.org/en-US/docs/Web/HTML/Element/q) - Pandoc \[\`Quoted\`](https&#x3A;//github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L262)

| Entity | type     | The name of the type and all descendant types. | string |
| ------ | -------- | ---------------------------------------------- | ------ |
| Entity | id       | The identifier for this item.                  | string |
| Mark   | content  | The content that is marked.                    |        |
| array  |          |                                                |        |
| Quote  | citation | The source of the quote.                       | string |

:::

The `Quote` schema represents inline quoted content.

# Examples

## Simple Quote

```json validate import=simple
{
  "type": "Quote",
  "content": [
    "If you wish to make an apple pie from scratch, you must first invent the universe. — Carl Sagan - Cosmos"
  ]
}
```

## With Attribution URI

```json validate import=attribution
{
  "type": "Quote",
  "content": [
    "If you wish to make an apple pie from scratch, you must first invent the universe. — Carl Sagan - Cosmos"
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

`Quote` is analogous to the HTML [`<q>` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q).

### Simple Quote

```html export=simple
<q
  >If you wish to make an apple pie from scratch, you must first invent the
  universe. — Carl Sagan - Cosmos</q
>
```

### With Attribution URI

```html export=attribution
<q
  cite="https://www.goodreads.com/quotes/32952-if-you-wish-to-make-an-apple-pie-from-scratch"
  >If you wish to make an apple pie from scratch, you must first invent the
  universe. — Carl Sagan - Cosmos</q
>
```

## JATS

`Quote` is analogous to the JATS [`<disp-quote>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/disp-quote.html) type, and the [`<attrib>` element](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/attrib.html) can be used for the `citation` field from the Stencila schema.

### Simple Quote

```jats export=simple
<p>‘If you wish to make an apple pie from scratch, you must first invent the universe. — Carl Sagan - Cosmos’</p>

```

### With Attribution URI

```jats export=attribution
<p>‘If you wish to make an apple pie from scratch, you must first invent the universe. — Carl Sagan - Cosmos’</p>

```

### mdast

`Quote` does not have mdast counterpart type, however please see the [`QuoteBlock`](/schema/QuoteBlock) schema for how to represent quotes in mdast.

### Simple Quote

```markdown export=simple
!quote[If you wish to make an apple pie from scratch, you must first invent the universe. — Carl Sagan - Cosmos]
```

### With Attribution URI

```markdown export=attribution
!quote[If you wish to make an apple pie from scratch, you must first invent the universe. — Carl Sagan - Cosmos](https://www.goodreads.com/quotes/32952-if-you-wish-to-make-an-apple-pie-from-scratch)
```

## OpenDocument

`Quote` does not have an analogous OpenDocument element.

## Pandoc

`Quote` is analogous to the Pandoc [`Quoted` type](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L262).

### Simple Quote

```pandoc export=simple
{
"blocks": [
{
"t": "Para",
"c": [
{
"t": "Quoted",
"c": [
{
"t": "SingleQuote"
},
[
{
"t": "Str",
"c": "If you wish to make an apple pie from scratch, you must first invent the universe. — Carl Sagan - Cosmos"
}
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

### With Attribution URI

```pandoc export=attribution
{
"blocks": [
{
"t": "Para",
"c": [
{
"t": "Quoted",
"c": [
{
"t": "SingleQuote"
},
[
{
"t": "Str",
"c": "If you wish to make an apple pie from scratch, you must first invent the universe. — Carl Sagan - Cosmos"
}
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
