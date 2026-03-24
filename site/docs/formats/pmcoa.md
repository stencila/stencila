---
title: PubMed Central OA Package
description: PubMed Central Open Access article packages
---

# Introduction

PubMed Central Open Access (PMCOA) packages bundle JATS XML articles with associated assets (figures, tables, supplementary files) as distributed by the [PMC Open Access service](https://pmc.ncbi.nlm.nih.gov/tools/oa-service/). These packages are a primary source of machine-readable scholarly articles in the biomedical sciences.

# Usage

Use the `.pmcoa` file extension, or the `--from pmcoa` option, when importing from a PMCOA package e.g.

```sh
stencila convert article.pmcoa doc.smd
```

> [!info]
> PMCOA is currently supported for import (decoding) only.

# Implementation

PMCOA decoding is implemented in the Rust crate [`codec-pmc`](https://github.com/stencila/stencila/blob/main/rust/codec-pmc). The codec extracts the JATS XML article and associated assets from the package archive. Structuring operations are applied to normalize citations, convert table images to rows, and convert math images to TeX.

# Limitations

- Export (encoding) to PMCOA format is not supported.
- The quality of the imported document depends on the quality of the source JATS XML, which varies between publishers.
- Supplementary data files within the package are not always linked to the main article content.
