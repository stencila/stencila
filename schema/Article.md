# Article

An `Article` type allows you to provide details about a document containing amongst other properties, the content as written prose, executable snippets of code, as well as images.

## Examples

These examples, based on [Marie Curie](https://en.wikipedia.org/wiki/Marie_Curie), illustrate alternative ways for specifying an `Article`.

### Simple

In its most basic form, an `Article` requires a `title` and a list of `authors`.

```json
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

### A more complete example

In addition to the properties demonstrated above, `content` is usually a key property to most articles.
The `content` property contains a list of [`Node`s](/schema/Node), meaning it can be contain any other valid node type.

```json
{
  "type": "Article",
  "title": "Introducing eLife’s first computationally reproducible article",
  "url": "https://elifesciences.org/labs/ad58f08d/introducing-elife-s-first-computationally-reproducible-article",
  "authors": ["Giuliano Maciocci", "Michael Aufreiter", "Nokome Bentley"],
  "content": [
    {
      "type": "Paragraph",
      "content": [
        {
          "type": "#text",
          "value": "In September 2017 eLife announced the start of the Reproducible Document Stack (RDS) project, a collaboration between Substance, Stencila and eLife to support the development of an open-source technology stack aimed at enabling researchers to publish reproducible manuscripts through online journals. Reproducible manuscripts enrich the traditional narrative of a research article with code, data and interactive figures that can be executed in the browser, downloaded and explored, giving readers a direct insight into the methods, algorithms and key data behind the published research."
        }
      ]
    },
    {
      "type": "Paragraph",
      "content": [
        {
          "type": "#text",
          "value": "Today eLife, in collaboration with "
        },
        {
          "type": "a"
        },
        {
          "type": "#text",
          "value": ", "
        },
        {
          "type": "a"
        },
        {
          "type": "#text",
          "value": " and Tim Errington, Director of Research ar the Center for Open Science, US, published its "
        },
        {
          "type": "a"
        },
        {
          "type": "#text",
          "value": ", based on one of Errington’s papers in the Reproducibility Project: Cancer Biology. This reproducible version of the article showcases some of what’s possible with the new RDS tools, and we invite researchers to explore the newly available opportunities to tell their story."
        }
      ]
    },
    {
      "type": "Heading",
      "depth": 3,
      "content": [
        {
          "type": "#text",
          "value": "Getting started with our reproducible article"
        }
      ]
    }
  ]
}
```

## Related

### JATS

`Article` is analagous, and structurally similar to, the JATS [`<article>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/article.html) element:

> A journal article `<article>` may be divided into three parts:
>
> 1. the `<front>` (the metadata or header information for the article, such as the title and the published date);
> 2. the `<body>` (textual and graphical content of the article); and
> 3. any `<back>` (ancillary information such as a glossary, reference list, or appendix).

In JATS the `<body>` element is the "Main textual portion of the document that conveys the narrative content."

### mdast

`Article.body` is analagous to the mdast [`Root`](https://github.com/syntax-tree/mdast#root) node type which "represents a document":

> Root can be used as the root of a tree, never as a child. Its content model is not limited to top-level content, but can contain any content with the restriction that all content must be of the same category.

### OpenDocument

`Article` is analagous to the OpenDocument `<office:document>` element is the root element in a single\* OpenDocument XML document:

> The [`<office:document>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1414998_253892949) element is the root element of a document in OpenDocument format which is represented as a single XML document. It contains the entire document.

A `<office:document>` has child elements,

- meta-data related `<office:meta>`, `<office:settings>`
- styles etc `<office:scripts>`, `<office:font-face-decls>`, `<office:styles>`, `<office:automatic-styles>`, `<office:master-styles>`
- content `<office:body>`

`Article.body` is analagous to `<office:body>` which can have child elements like `<office:drawing>`, `<office:presentation>`, `<office:spreadsheet>`, `<office:text>`. The primary difference is that `Article.body` can only contain named `Sheet`s (which themselves contain text or spreadsheet type documents).

Other properties of `Article` are analagous to those in [`<office:meta>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415014_253892949).

\*: In a multi-XML document OpenDocument properties are group together e.g. content in `<office:document-content>` and styles in `<office:document-styles>`.
