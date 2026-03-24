---
title: R+LaTeX
description: Sweave/knitr Rnw format for literate programming
---

# Introduction

Rnw (R NoWeb) files combine LaTeX markup with embedded R code chunks, following the [Sweave](https://leisch.usgs.gov/Sweave/) and [knitr](https://yihui.org/knitr/) literate programming conventions. This format is commonly used in R-based reproducible research workflows.

# Usage

Use the `.rnw` file extension, or the `--to rnw` or `--from rnw` options, when converting to/from Rnw e.g.

```sh
stencila convert doc.smd doc.rnw
```

# Implementation

Rnw support is implemented in the Rust crate [`codec-rnw`](https://github.com/stencila/stencila/blob/main/rust/codec-rnw).

# Limitations

- Conversion is lossy for Stencila node types that have no LaTeX or R code chunk equivalent.
- Complex knitr chunk options may not be fully mapped to Stencila code chunk properties.
- Stencila does not process Sweave/knitr directives natively; R code chunks are represented as Stencila code chunks.
