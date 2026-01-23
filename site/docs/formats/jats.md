---
title: JATS
description: Journal Article Tag Suite
---
# Introduction

The [JATS (Journal Article Tag Suite)](https://jats.nlm.nih.gov/) is an XML format that can be used to tag and describe scientific articles. It was developed by the NIH and has been adopted by several journals as a de facto standard for describing journal articles.

# Usage

Use the `.jats.xml` file extension, or the `--to jats` or `--from jats` options, when converting to/from JATS e.g.

```sh
stencila convert doc.smd doc.jats.xml
```

By default, the encoded JATS is un-indented. Use the `--pretty` option for indented XML but note that this may affect whitespace.

# Implementation

Stencila supports bi-directional conversion between Stencila documents and JATS. Parsing of JATS is built on top of the [`quick-xml`](https://crates.io/crates/quick-xml) Rust crate.

# Notes

- JATS is XML-based and primarily targets scholarly articles.
- Use `--pretty` with care because whitespace can be significant in XML.
