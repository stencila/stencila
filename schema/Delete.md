---
title: Delete
authors: []
---

include: ../built/Delete.schema.md
:::
Content that is marked for deletion

| Thing | type           | The name of the type and all descendant types. | string |
| ----- | -------------- | ---------------------------------------------- | ------ |
| Thing | id             | The identifier for this item.                  | string |
| Thing | alternateNames | Alternate names (aliases) for the item.        | array  |
| Thing | description    | A description of the item.                     | string |
| Thing | meta           | Metadata associated with this item.            | object |
| Thing | name           | The name of the item.                          | string |
| Thing | url            | The URL of the item.                           | string |
| Mark  | content        | The content that is marked.                    |        |
| array |                |                                                |        |
:::

# Examples

To illustrate how `Delete` nodes are encoded in alternative formats, we'll use the following example, in context, within a `Paragraph`:

```json import=inpara
{
  "type": "Paragraph",
  "content": [
    "The following is ",
    {
      "type": "Delete",
      "content": ["marked for deletion"]
    },
    "."
  ]
}
```

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section
:::

## HTML

HTML natively supports `Delete` nodes with the [`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del) element.

```html export=inpara
<p>The following is <del>marked for deletion</del>.</p>
```

## Markdown

Most Markdown parsers support the use of tildes (`~`) to mark content for deletion. For example, MDAST also has a [`Delete`](https://github.com/syntax-tree/mdast#delete) node type, which renders the above example like this:

```md export=inpara
The following is ~~marked for deletion~~.
```

## Pandoc

The equivalent of `Delete` in Pandoc is the [`Strikeout`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L258) element. The above example in Pandoc JSON:

```json export=inpara to=pandoc
{
  "blocks": [
    {
      "t": "Para",
      "c": [
        {
          "t": "Str",
          "c": "The following is "
        },
        {
          "t": "Strikeout",
          "c": [
            {
              "t": "Str",
              "c": "marked for deletion"
            }
          ]
        },
        {
          "t": "Str",
          "c": "."
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
