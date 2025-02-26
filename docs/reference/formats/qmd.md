---
config:
  publish:
    ghost:
      slug: qmd-format
      state: publish
      tags:
      - '#doc'
      - Formats
      type: post
description: Quarto Markdown
title: qmd
---

# Introduction

**File Extension:** `.qmd` - Used when converting or exporting Stencila documents to CBORZST format.

The [Quarto Markdown](https://quarto.org/docs/authoring/markdown-basics.html) is a special flavor of Markdown supported by Stencila and used by those that use the Quarto document publishing system.

# Implementation

Stencila support bi-directional conversion between Stencila documents and QMD powered by the CommonMark compliant [`markdown`](https://crates.io/crates/markdown) crate.

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
