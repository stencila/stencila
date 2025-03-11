---
title: pandoc
description: Pandoc JSON
config:
  publish:
    ghost:
      slug: pandoc
      tags:
        - "#docs"
        - Formats
---

# Introduction

Pandoc is a document converter that allows you to translate content between a wide range of markup formats, including Markdown, HTML, LaTeX. It is widely used in academic and technical publishing.

[Pandoc JSON](https://hackage.haskell.org/package/pandoc-types-1.23.1/docs/Text-Pandoc-JSON.html) is a JSON representation of Pandoc's internal document element types.

Stencila delegates conversion to/from several formats (e.g. [DOCX](../formats/docx.md)) via Pandoc JSON.

# Usage

You are unlikely to want to use Pandoc JSON format directly, but if you do (e.g. for debugging), use the `.pandoc` file extension, or the `--to pandoc` or `--from pandoc` options e.g.

```sh
stencila convert doc.smd doc.pandoc
```

# Implementation

Stencila support bi-directional conversion between Stencila documents and Pandoc JSON powered by the [`pandoc_types`](https://crates.io/crates/pandoc_types) Rust crate.

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
