---
title: HTML
description: HyperText Markup Language
---

# Introduction

HTML (HyperText Markup Language) is the standard markup language for structuring and displaying content on the web. Stencila provides bi-directional conversion between Stencila documents and HTML.

# Usage

Use the `.html` file extension, or the `--to html` or `--from html` options, when converting to/from HTML e.g.

```sh
stencila convert doc.smd doc.html
```

> [!note]
> Stencila also provides a [DOM HTML](../dom) format intended for document previewing, and an [Email HTML](../email) format optimized for email clients. Plain HTML is best suited for general-purpose web publishing.

> [!warning]
> Stencila's HTML support is in alpha status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila supports bi-directional conversion between Stencila documents and HTML. Parsing of HTML uses the [`tl`](https://crates.io/crates/tl) Rust crate. Encoding to HTML is implemented via derived trait methods in [`codec-html-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-html-trait).

# Limitations

- Decoding from HTML is lossy for elements that have no Stencila Schema equivalent (e.g. `<div>` layout, `<form>` elements, custom attributes).
- CSS styling and JavaScript are not preserved during import.
- HTML is always decoded as an `Article`; other root node types are not inferred from HTML structure.
