---
config:
  publish:
    ghost:
      slug: text-format
      state: publish
      tags:
      - '#doc'
      - Formats
      type: post
description: Plain Text
title: text
---

## Introduction

**File Extension:** `.text` - Used when converting or exporting Stencila documents to plain text format.

The text format is a lossy output format which can be used to convert many document types to plain text representations. 

## Implementation

Stencila support lossless, bi-directional conversion between Stencila documents and CBOR powered by [`ciborium`](https://crates.io/crates/ciborium).

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
