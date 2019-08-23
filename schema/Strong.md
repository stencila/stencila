---
title: Strong
authors: []
---

include: ../public/Strong.schema.md
:::
Strongly emphasised content. Analagous to, - JATS \[\`&lt;bold>\`](https&#x3A;//jats.nlm.nih.gov/archiving/tag-library/1.1/element/bold.html) - HTML \[\`&lt;strong>\`](https&#x3A;//developer.mozilla.org/en-US/docs/Web/HTML/Element/strong) - MDAST \[\`Strong\`](https&#x3A;//github.com/syntax-tree/mdast#strong) - Pandoc \[\`Strong\`](https&#x3A;//github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L257)

| Entity | type    | The name of the type and all descendant types. | string |
| ------ | ------- | ---------------------------------------------- | ------ |
| Entity | id      | The identifier for this item.                  | string |
| Mark   | content | The content that is marked.                    |        |
| array  |         |                                                |        |

:::

The `Strong` schema represents strongly emphasised content. It can contain any valid [`InlineContent`](/schema/InlineContent) nodes.

# Examples

## Simple

```json validate import=simple
{
  "type": "Strong",
  "content": ["Some important information"]
}
```

## Nested types

```json validate import=nested
{
  "type": "Strong",
  "content": [
    "Some ",
    { "type": "Delete", "content": ["important"] },
    "essential information"
  ]
}
```

# Encodings

include: ../docs/type-encodings-intro.md
:::
This section describes common encodings for this node type. These samples are generated from the above examples by [Encoda](https://stencila.github.io/encoda), but you can also author them in each format.
:::

## JATS

`Strong` is analogous, and structurally similar to, the JATS [`<bold>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/aut-elem-sec-intro.html) type.

### Simple

```jats export=simple
<p><bold>Some important information</bold></p>

```

### Nested types

```jats export=nested
<p><bold>Some <strike>important</strike>essential information</bold></p>

```

## mdast

`Strong` is analogous to the mdast [`Strong`](https://github.com/syntax-tree/mdast#strong) node type.

### Simple

```markdown export=simple
**Some important information**
```

### Nested types

```markdown export=nested
**Some ~~important~~essential information**
```

## OpenDocument

`Strong` is similar to the OpenDocument [`<style:font-adornments>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1417910_253892949) attribute.

[//]: # 'WIP: Some <strong> elements are ending up in the generated HTML doc making everything bold'
