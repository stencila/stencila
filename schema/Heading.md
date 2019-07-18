# Formats

## JSON

To illustrate how `Heading` nodes are represented in alternative formats, we'll use the following example.

```json import=heading
{
  "type": "Heading",
  "depth": 2,
  "content": ["Secondary Heading"]
}
```

For compatibility with HTML, only integer depths in the range 1–6 are supported.

## HTML

HTML supports `Heading` nodes with the [`<h1>` to `<h6>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1) elements.

```html export=heading

```

## JATS

JATS lacks a depth attribute so this is lost on conversion, the [`<title>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/title.html) element is used.

```jats export=heading

```

## YAML

```yaml export=heading
```

## Markdown

Markdown headings are denoted by a number of hashes (`#`) equal to the depth.

```markdown export=heading
```

## Pandoc

The equivalent of `Heading` in Pandoc is the [`Header`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L233) element. The above example in Pandoc JSON:

```pandoc export=heading
```
