---
title: TEI
description: Text Encoding Initiative XML, as produced by GROBID
---

# Introduction

The [TEI (Text Encoding Initiative)](https://tei-c.org/) is an XML format for encoding texts. [GROBID](https://github.com/grobidOrg/grobid), which extracts the structure of scholarly articles from PDFs, writes its results as TEI, and [OpenAlex](https://openalex.org/) serves GROBID's TEI for many works. Stencila decodes GROBID's full-text TEI so that articles that are only available as PDFs can be converted in the same way as those available as JATS.

# Usage

Use the `.tei.xml` file extension, or the `--from tei` option, when importing from TEI e.g.

```sh
stencila convert paper.tei.xml paper.json
```

Gzip compressed TEI (e.g. `paper.tei.xml.gz`, as OpenAlex distributes it) is decompressed when decoding. Use `--input-losses` to report what could not be decoded:

```sh
stencila convert paper.tei.xml paper.json --input-losses losses.yaml
```

> [!info]
> TEI is currently supported for import (decoding) only.

# Implementation

TEI decoding is implemented in the Rust crate [`codec-tei`](https://github.com/stencila/stencila/blob/main/rust/codec-tei), using the [`roxmltree`](https://crates.io/crates/roxmltree) crate to parse the XML. GROBID's TEI is decoded to the same Stencila nodes that the JATS codec decodes the same content to: the header to the article's title, authors (with their affiliations, emails and ORCIDs), identifiers, journal, funders, keywords and abstract; the divisions of the body to sections, nested according to GROBID's numbering of their headings; references to bibliographic entries to citations; figures and tables (including their cells) to figures and tables; and the bibliography to references.

Consecutive abstracts in `profileDesc` with the same `xml:lang` are joined in source order, preserving their blocks. Whitespace, comments and processing instructions between abstracts do not break a run; any other element or text does, as does an abstract in another language, even an empty one. The first nonempty group becomes the article's `abstract`; later groups become nested articles in `parts`, each with its own `abstract` and, when specified, `inLanguage` metadata. Language tags are inherited from ancestor elements (an empty `xml:lang` value resets them) and compared without regard to case. Abstracts with no language are joined with each other but kept separate from those with a specified language.

An article has no property for the language of its main abstract, so that language is recorded as lost: on the ancestor that sets it when it is inherited (e.g. GROBID's `teiHeader`), otherwise on the abstracts themselves. The `inLanguage` of the parts is extra metadata that the JATS encoder does not yet write, so encoding them to JATS records it as a loss.

## Page coordinates

GROBID and ScienceBeam can give the coordinates of elements on the pages of the PDF (their `coords`). These are information about the source rather than the content of the article, so they are not kept on the decoded nodes. Instead, they are returned alongside the article as a page map, from each node to its boxes on the pages, in the same way that decoding a text format returns a mapping from each node to its position in the text. Where the boxes are of a part of a node, the part is named, for example `persName` for the name of an author, `graphic` for the graphic of a figure, and `marker` for where a footnote is referenced.

`stencila compare` includes the page maps of the documents compared in its JSON and YAML outputs, under `pages.left` and `pages.right`, indexed by node path, so that a box on a page can be related to the comparison:

```sh
stencila compare reference.jats.xml parse.tei.xml comparison.json
```

# Limitations

- GROBID gives no image files for figures, only the coordinates of each figure on the pages of the PDF. Figures are decoded without an image, with the coordinates in the page map, and each missing image is reported as a loss. A figure's notes are kept after its images.
- GROBID gives formulas as the text extracted from the PDF, not TeX or MathML. Formulas are decoded as math with the language `text`, and reported as a loss.
- The quality of the decoded article depends on GROBID's extraction from the PDF, which can, for example, take lines of a figure or table to be headings, or miss tables.
- Export (encoding) to TEI is not supported.
