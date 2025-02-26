---
config:
  publish:
    ghost:
      slug: odt-format
      state: publish
      tags:
      - '#doc'
      - Formats
      type: post
description: Open Document Template
title: ODT
---

## Introduction

**File Extension:** `.odt` - Used when converting or exporting Stencila documents to odt format.

The [ODT](https://docs.oasis-open.org/office/v1.1/OS/OpenDocument-v1.1-html/OpenDocument-v1.1.html) format is a serialization format is a convenient when you want to pass open documents to and from word processors.

## Implementation

It is made possible in Stencila by using the intermediate Stencila format [pandoc](docs/format-pandoc), which converts documents to [pandoc-json](https://hackage.haskell.org/package/pandoc-types-1.23.1/docs/Text-Pandoc-JSON.html).

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
