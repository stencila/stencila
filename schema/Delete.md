# Formats

## JSON

To illustrate how `Delete` nodes are represented in alternative formats, we'll use the following example, in context, within a `Paragraph`:

```json import=inpara
{
  "type": "Paragraph",
  "content": [
    "The following content is ",
    {
      "type": "Delete",
      "content": ["marked for deletion"]
    }
  ]
}
```

## HTML

HTML natively supports `Delete` nodes with the [`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del) element.

```html export=inpara
```

## YAML

```yaml export=inpara
```

## Markdown

Most Markdown parsers support the use of tildes (`~`) to mark content for deletion. For example, MDAST also has a [`Delete`](https://github.com/syntax-tree/mdast#delete) node type, which renders the above example like this:

```md export=inpara
```

## Pandoc

The equivalent of `Delete` in Pandoc is the [`Strikeout`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L258) element. The above example in Pandoc JSON:

```pandoc export=inpara format=pandoc
```
