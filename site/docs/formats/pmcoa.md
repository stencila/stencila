---
title: PubMed Central OA Package
description: PubMed Central Open Access package
---
# Introduction

PubMed Central Open Access (PMCOA) packages bundle JATS XML with associated assets for scholarly articles.

# Usage

Use the `.pmcoa` file extension, or the `--from pmcoa` option, when converting from a PMCOA package e.g.

```sh
stencila convert article.pmcoa doc.smd
```

# Implementation

PMCOA decoding is implemented in the Rust crate [`codec-pmc`](https://github.com/stencila/stencila/blob/main/rust/codec-pmc).

# Notes

- PMCOA is currently supported for import only.
- The package typically contains JATS XML plus figures and supplementary files.
