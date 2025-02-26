---
config:
  publish:
    ghost:
      slug: docx-format
      state: publish
      tags:
      - '#doc'
      - Formats
      type: post
description: Word Document XML format
title: docx
---

# Introduction

**File Extension:** `.docx` - Used when converting or exporting Stencila documents to docx format.

The [DOCX format](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-docx/d683fa62-8042-4360-a824-b79045a6aabd) is a format useful for sharing documents with others in a format they may be familiar and comfortable with. 


# Implementation

It is made possible in Stencila by using the intermediate Stencila format [pandoc](docs/format-pandoc), which converts documents to [pandoc-json](https://hackage.haskell.org/package/pandoc-types-1.23.1/docs/Text-Pandoc-JSON.html).

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->



<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
