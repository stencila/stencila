# Cite

**A reference to a CreativeWork that is cited in another CreativeWork.**

A `Cite` node is used within a [`CreativeWork`](./CreativeWork), usually an [`Article`](./Article), to refer to an other `CreativeWork`. Often a `Cite` will be associated with other citations, in a [`CiteGroup`](./CiteGroup).

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name           | `@id`                                                                     | Type                                                                                         | Description                                                                                           | Inherited from      |
| -------------- | ------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | ------------------- |
| **target**     | [stencila:target](https://schema.stenci.la/target.jsonld)                 | string                                                                                       | The target of the citation (URL or reference ID).                                                     | [Cite](Cite.md)     |
| citationIntent | [stencila:citationIntent](https://schema.stenci.la/citationIntent.jsonld) | Array of [CitationIntentEnumeration](CitationIntentEnumeration.md)                           | The type/s of the citation, both factually and rhetorically.                                          | [Cite](Cite.md)     |
| citationMode   | [stencila:citationMode](https://schema.stenci.la/citationMode.jsonld)     | 'Parenthetical', 'Narrative', 'NarrativeAuthor', 'NarrativeYear', 'normal', 'suppressAuthor' | Determines how the citation is shown within the surrounding text. See note [1](#notes).               | [Cite](Cite.md)     |
| citationPrefix | [stencila:citationPrefix](https://schema.stenci.la/citationPrefix.jsonld) | string                                                                                       | Text to show before the citation. See note [2](#notes).                                               | [Cite](Cite.md)     |
| citationSuffix | [stencila:citationSuffix](https://schema.stenci.la/citationSuffix.jsonld) | string                                                                                       | Text to show after the citation. See note [3](#notes).                                                | [Cite](Cite.md)     |
| content        | [stencila:content](https://schema.stenci.la/content.jsonld)               | Array of [InlineContent](InlineContent.md)                                                   | Optional structured content/text of this citation.                                                    | [Cite](Cite.md)     |
| id             | [schema:id](https://schema.org/id)                                        | string                                                                                       | The identifier for this item.                                                                         | [Entity](Entity.md) |
| meta           | [stencila:meta](https://schema.stenci.la/meta.jsonld)                     | object                                                                                       | Metadata associated with this item.                                                                   | [Entity](Entity.md) |
| pageEnd        | [schema:pageEnd](https://schema.org/pageEnd)                              | integer _or_ string                                                                          | The page on which the work ends; for example "138" or "xvi".                                          | [Cite](Cite.md)     |
| pageStart      | [schema:pageStart](https://schema.org/pageStart)                          | integer _or_ string                                                                          | The page on which the work starts; for example "135" or "xiii".                                       | [Cite](Cite.md)     |
| pagination     | [schema:pagination](https://schema.org/pagination)                        | string                                                                                       | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55". | [Cite](Cite.md)     |

## Notes

1. **citationMode** : There are two main citation modes: parenthetical and narrative (a.k.a textual). See https://apastyle.apa.org/style-grammar-guidelines/citations/basic-principles/parenthetical-versus-narrative for an explanation. This property is optional and tools are recommended to assume `parenthetical` if missing. Narrative citations will usually be of form "As noted by Smith (1992)," but `narrative-author` allows for "In the early nineties, Smith noted" and `narrative-year` allows for "As noted by Smith in 1992 and 1993". Pandoc's `CitationMode` enumeration has `Normal` (for `parenthetical`), `AuthorInText` (for `textual`), and `SuppressAuthor` (for `textual-year`). See https://github.com/jgm/pandoc-types/blob/0158cd0e2a2ca9d6f14389a1a57bc64cab45a7dd/src/Text/Pandoc/Definition.hs#L353. LaTeX's `natbib` package has `\citep{}` (for `parenthetical`), `\citet{}` (for `textual`), `\citeauthor{}` (for `textual-author`), `\citeyear{}` (for `textual-year`). See https://www.overleaf.com/learn/latex/Natbib_citation_styles.
2. **citationPrefix** : Previously this was name `prefix` but for consistency with `citationMode` and `honorificPrefix`, to avoid ambiguity with other prefixes was renamed to `citationPrefix`.
3. **citationSuffix** : See comment on `citationPrefix` regarding naming.

## Examples

```json
{
  "type": "Article",
  "title": "An example of using the Cite node type",
  "authors": [
    {
      "type": "Person",
      "givenNames": ["Joe"],
      "familyNames": ["Bloggs"]
    }
  ],
  "content": [
    {
      "type": "Paragraph",
      "content": [
        "A citation of ",
        {
          "type": "Cite",
          "target": "some-one-else-1991"
        },
        "."
      ]
    }
  ],
  "references": [
    {
      "type": "Article",
      "id": "some-one-else-1991",
      "title": "Another article by someone else",
      "authors": [
        {
          "type": "Person",
          "givenNames": ["Some", "One"],
          "familyNames": ["Else"]
        }
      ],
      "datePublished": "1991"
    }
  ]
}
```

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Cite.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Cite.schema.json)
- Python [`class Cite`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Cite)
- TypeScript [`interface Cite`](https://stencila.github.io/schema/ts/docs/interfaces/cite.html)
- R [`class Cite`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Cite`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Cite.html)

## Source

This documentation was generated from [Cite.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Cite.schema.yaml).
