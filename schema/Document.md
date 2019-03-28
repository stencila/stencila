## Related

### JATS

`Document` is analagous, and structurally similar to, the JATS [`<article>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/article.html) element:

> A journal article `<article>` may be divided into three parts:
>
> 1. the `<front>` (the metadata or header information for the article, such as the title and the published date);
> 2. the `<body>` (textual and graphical content of the article); and
> 3. any `<back>` (ancillary information such as a glossary, reference list, or appendix).

In JATS the `<body>` element is the "Main textual portion of the document that conveys the narrative content."

### mdast

`Document.body` is analagous to the mdast [`Root`](https://github.com/syntax-tree/mdast#root) node type which "represents a document":

> Root can be used as the root of a tree, never as a child. Its content model is not limited to top-level content, but can contain any content with the restriction that all content must be of the same category.

### OpenDocument

`Document` is analagous to the OpenDocument `<office:document>` element is the root element in a single\* OpenDocument XML document:

> The [`<office:document>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1414998_253892949) element is the root element of a document in OpenDocument format which is represented as a single XML document. It contains the entire document.

A `<office:document>` has child elements,

- meta-data related `<office:meta>`, `<office:settings>`
- styles etc `<office:scripts>`, `<office:font-face-decls>`, `<office:styles>`, `<office:automatic-styles>`, `<office:master-styles>`
- content `<office:body>`

`Document.body` is analagous to `<office:body>` which can have child elements like `<office:drawing>`, `<office:presentation>`, `<office:spreadsheet>`, `<office:text>`. The primary difference is that `Document.body` can only contain named `Sheet`s (which themselves contain text or spreadsheet type documents).

Other properties of `Document` are analagous to those in [`<office:meta>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415014_253892949).

\*: In a multi-XML document OpenDocument properties are group together e.g. content in `<office:document-content>` and styles in `<office:document-styles>`.
