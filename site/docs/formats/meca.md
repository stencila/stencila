---
title: MECA
description: Manuscript Exchange Common Approach package
---

# Introduction

[MECA (Manuscript Exchange Common Approach)](https://www.niso.org/standards-committees/meca) is a NISO-standardized package format for exchanging manuscripts and related files (figures, supplementary data, metadata) between scholarly publishing systems. A MECA package bundles a JATS XML article with its associated assets into a single archive.

# Usage

Use the `.meca` file extension, or the `--from meca` option, when importing from a MECA package e.g.

```sh
stencila convert article.meca doc.smd
```

> [!info]
> MECA is currently supported for import (decoding) only.

# Implementation

MECA decoding is implemented in the Rust crate [`codec-meca`](https://github.com/stencila/stencila/blob/main/rust/codec-meca). The codec extracts the primary article content (typically JATS XML) from the package and decodes it using the [JATS](../jats) codec.

# Limitations

- Export (encoding) to MECA is not supported.
- Only the primary article content is extracted; supplementary materials and review metadata may not be fully imported.
- The MECA manifest must follow the standard structure for the codec to locate the article file.
