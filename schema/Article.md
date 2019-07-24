---
title: Article
authors: []
---

An `Article` type allows you to provide details about a document containing amongst other properties, the content as written prose, executable snippets of code, as well as images.

include: ../built/Article.schema.md
:::
| Thing | type | The name of the type and all descendant types. | string |
| ------------ | -------------- | ----------------------------------------------------------------------------- | ------ |
| Thing | id | The identifier for this item. | string |
| Thing | alternateNames | Alternate names (aliases) for the item. | array |
| Thing | description | A description of the item. | string |
| Thing | meta | Metadata associated with this item. | object |
| Thing | name | The name of the item. | string |
| Thing | url | The URL of the item. | string |
| CreativeWork | authors | The authors of this this creative work. | array |
| CreativeWork | citations | Citations or references to other creative works, such as another publication, | |

web page, scholarly article, etc. | array | | CreativeWork | content | The structured content of this creative work c.f. property \`text\`. | array | | CreativeWork | dateCreated | Date/time of creation. | | | CreativeWork | dateModified | Date/time of most recent modification. | | | CreativeWork | datePublished | Date of first publication. | | | CreativeWork | editors | Persons who edited the CreativeWork. | array | | CreativeWork | funders | Person or organisation that funded the CreativeWork. | array | | CreativeWork | isPartOf | An item or other CreativeWork that this CreativeWork is a part of. | | | CreativeWork | licenses | License documents that applies to this content, typically indicated by URL. | array | | CreativeWork | parts | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | array | | CreativeWork | publisher | A publisher of the CreativeWork. | | | CreativeWork | text | The textual content of this creative work. | string | | CreativeWork | title | | string | | CreativeWork | version | | | | Article | environment | The computational environment in which the document should be executed. | |
:::

# Formats

These examples, based on [Marie Curie](https://en.wikipedia.org/wiki/Marie_Curie), illustrate alternative ways for specifying an `Article`.

## JSON (Simple)

In its most basic form, an `Article` requires a `title` and a list of `authors`.

```json import=simple
{
  "type": "Article",
  "title": "Recherches sur les substances radioactives",
  "authors": [
    {
      "type": "Person",
      "honorificPrefix": "Dr",
      "givenNames": ["Marie", "Skłodowska"],
      "familyNames": ["Curie"],
      "honorificSuffix": "PhD"
    }
  ]
}
```

## JSON (More Complex)

In addition to the properties demonstrated above, `content` is usually a key property to most articles. The `content` property contains a list of [`Node`s](/schema/Node), meaning it can be contain any other valid node type.

```json import=complex
{
  "type": "Article",
  "title": "Introducing eLife’s first computationally reproducible article",
  "url": "https://elifesciences.org/labs/ad58f08d/introducing-elife-s-first-computationally-reproducible-article",
  "authors": ["Giuliano Maciocci", "Michael Aufreiter", "Nokome Bentley"],
  "content": [
    {
      "type": "Paragraph",
      "content": [
        "In September 2017 eLife announced the start of the Reproducible Document Stack (RDS) project, a collaboration between Substance, Stencila and eLife to support the development of an open-source technology stack aimed at enabling researchers to publish reproducible manuscripts through online journals. Reproducible manuscripts enrich the traditional narrative of a research article with code, data and interactive figures that can be executed in the browser, downloaded and explored, giving readers a direct insight into the methods, algorithms and key data behind the published research."
      ]
    },
    {
      "type": "Paragraph",
      "content": [
        "Today eLife, in collaboration with ",
        {
          "type": "Link",
          "target": "http://substance.io/",
          "content": ["Substance"]
        },
        ", ",
        {
          "type": "Link",
          "target": "https://stenci.la/",
          "content": ["Stencila"]
        },
        " and Tim Errington, Director of Research ar the Center for Open Science, US, published its ",
        {
          "type": "Link",
          "target": "https://elifesci.org/reproducible-example",
          "content": ["first reproducible article"]
        },
        ", based on one of Errington’s papers in the Reproducibility Project: Cancer Biology. This reproducible version of the article showcases some of what’s possible with the new RDS tools, and we invite researchers to explore the newly available opportunities to tell their story."
      ]
    }
  ]
}
```

## JATS

`Article` is analagous, and structurally similar to, the JATS [`<article>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/article.html) element:

> A journal article `<article>` may be divided into three parts:
>
> 1.  the `<front>` (the metadata or header information for the article, such as the title and the published date);
> 2.  the `<body>` (textual and graphical content of the article); and
> 3.  any `<back>` (ancillary information such as a glossary, reference list, or appendix).

In JATS the `<body>` element is the "Main textual portion of the document that conveys the narrative content."

## mdast

`Article.body` is analagous to the mdast [`Root`](https://github.com/syntax-tree/mdast#root) node type which "represents a document":

> Root can be used as the root of a tree, never as a child. Its content model is not limited to top-level content, but can contain any content with the restriction that all content must be of the same category.

### Simple Example

```markdown export=simple
---
title: Recherches sur les substances radioactives
authors:
  - type: Person
    honorificPrefix: Dr
    givenNames:
      - Marie
      - Skłodowska
    familyNames:
      - Curie
    honorificSuffix: PhD
---
```

### Complex Example

```markdown export=complex
---
title: Introducing eLife’s first computationally reproducible article
url: >-
  https://elifesciences.org/labs/ad58f08d/introducing-elife-s-first-computationally-reproducible-article
authors:
  - type: Person
    givenNames:
      - Giuliano
    familyNames:
      - Maciocci
  - type: Person
    givenNames:
      - Michael
    familyNames:
      - Aufreiter
  - type: Person
    givenNames:
      - Nokome
    familyNames:
      - Bentley
---

In September 2017 eLife announced the start of the Reproducible Document Stack (RDS) project, a collaboration between Substance, Stencila and eLife to support the development of an open-source technology stack aimed at enabling researchers to publish reproducible manuscripts through online journals. Reproducible manuscripts enrich the traditional narrative of a research article with code, data and interactive figures that can be executed in the browser, downloaded and explored, giving readers a direct insight into the methods, algorithms and key data behind the published research.

Today eLife, in collaboration with [Substance](http://substance.io/), [Stencila](https://stenci.la/) and Tim Errington, Director of Research ar the Center for Open Science, US, published its [first reproducible article](https://elifesci.org/reproducible-example), based on one of Errington’s papers in the Reproducibility Project: Cancer Biology. This reproducible version of the article showcases some of what’s possible with the new RDS tools, and we invite researchers to explore the newly available opportunities to tell their story.
```

## OpenDocument

`Article` is analagous to the OpenDocument `<office:document>` element is the root element in a single**\*** OpenDocument XML document:

> The [`<office:document>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1414998_253892949) element is the root element of a document in OpenDocument format which is represented as a single XML document. It contains the entire document.

A `<office:document>` has child elements,

- meta-data related `<office:meta>`, `<office:settings>`
- styles etc `<office:scripts>`, `<office:font-face-decls>`, `<office:styles>`, `<office:automatic-styles>`, `<office:master-styles>`
- content `<office:body>`

`Article.body` is analagous to `<office:body>` which can have child elements like `<office:drawing>`, `<office:presentation>`, `<office:spreadsheet>`, `<office:text>`. The primary difference is that `Article.body` can only contain named `Sheet`s (which themselves contain text or spreadsheet type documents).

Other properties of `Article` are analagous to those in [`<office:meta>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415014_253892949).

**\***: In a multi-XML document OpenDocument properties are group together e.g. content in `<office:document-content>` and styles in `<office:document-styles>`.

### Simple Example

This [`odt`](article-simple-ex1.out.odt) file was generated from the simple example.

### Complex Example

This [`odt`](article-complex-ex1.out.odt) file was generated from the simple example.
