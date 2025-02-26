---
config:
  publish:
    ghost:
      slug: smd-format
      state: publish
      tags:
      - '#doc'
      - Formats
      type: post
description: Stencila Markdown
title: smd
---

# Introduction

**File Extension:** `.smd` - Used when converting or exporting Stencila documents to smd format.

The [Stencila Markdown](docs/smd) format is a serialization format which strives to support as many features of the [Stencila Schema](docs/schema) as possible with special Markdown sytnax.

# Implementation

Stencila support bi-directional conversion between Stencila documents and Stencila Markdown powered by the CommonMark [`markdown`](https://crates.io/crates/markdown) crate.

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
