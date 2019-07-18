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

```

## JATS

The JATS equivalent to `Emphasis` is the [`<italic>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/italic.html) element.

```jats export=inpara

```

## YAML

```yaml export=inpara
```

## Markdown

Emphasis in Markdown can be achieved with underscores (`_`) or asterisks (`*`). See also the [MDAST reference](https://github.com/syntax-tree/mdast#emphasis).

```md export=inpara
```

## Pandoc

The equivalent of `Emphasis` in Pandoc is the [`Emph`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L256) element. The above example in Pandoc JSON:

```pandoc export=inpara format=pandoc
```
