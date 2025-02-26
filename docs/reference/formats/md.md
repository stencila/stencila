---
config:
  publish:
    ghost:
      slug: md-format
      state: publish
      tags:
      - '#doc'
      - Formats
      type: post
description: Vanilla Markdown
title: Markdown
---

## Introduction

**File Extension:** `.md` - Used when converting or exporting Stencila documents to markdown format.

The Markdown [CommonMark](https://spec.commonmark.org/) format is a serialization format is a convenient serialization format that can be used when basic plain text features are needed. Other Markdown formats such as [Stencila Markdown](/docs/smd-format), [MyST Markdown](/docs/myst-format), and [Quarto Markdown](/docs/qmd-format) have various ways in which [Stencila Schema](/docs/schema) items can be included in Markdown files in ways that can be streamlined with existing tools and workflows you might use.

## Implementation

Stencila support variably lossy bi-directional conversion between Stencila documents and Markdown powered by the Rust CommonMark compliant [`markdown`](https://crates.io/crates/markdown) crate.

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
