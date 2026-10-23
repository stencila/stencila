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

# Limitations

- GROBID gives no image files for figures, only the coordinates of each figure on the pages of the PDF. Figures are decoded without content, with the coordinates kept as `coords` identifiers, and each missing image is reported as a loss.
- GROBID gives formulas as the text extracted from the PDF, not TeX or MathML. Formulas are decoded as math with the language `text`, and reported as a loss.
- The quality of the decoded article depends on GROBID's extraction from the PDF, which can, for example, take lines of a figure or table to be headings, or miss tables.
- Export (encoding) to TEI is not supported.
