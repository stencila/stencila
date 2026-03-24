---
title: JATS
description: Journal Article Tag Suite
---

# Introduction

The [JATS (Journal Article Tag Suite)](https://jats.nlm.nih.gov/) is an XML-based format for tagging and describing scientific articles. Developed by the National Library of Medicine (NLM), it has been widely adopted by publishers and archives as a standard for scholarly article markup.

# Usage

Use the `.jats.xml` file extension, or the `--to jats` or `--from jats` options, when converting to/from JATS e.g.

```sh
stencila convert doc.smd doc.jats.xml
```

By default, the encoded JATS XML is un-indented for compactness. Use the `--pretty` option for indented output, but note that this can introduce whitespace that may affect rendering in some XML processors.

# Implementation

Stencila supports bi-directional conversion between Stencila documents and JATS. Parsing of JATS XML uses the [`roxmltree`](https://crates.io/crates/roxmltree) Rust crate with encoding powered by [`quick-xml`](https://crates.io/crates/quick-xml). Per-node-type JATS encoding is derived via the [`codec-jats-trait`](https://github.com/stencila/stencila/blob/main/rust/codec-jats-trait) and [`codec-jats-derive`](https://github.com/stencila/stencila/blob/main/rust/codec-jats-derive) crates.

# Limitations

- JATS is designed for scholarly articles. Non-article document types may not map well to JATS structure.
- Stencila-specific node types (e.g. executable code chunks, parameters, styled blocks) have no JATS equivalent and are lost during export.
- Some JATS elements (e.g. `<supplementary-material>`, `<ext-link>` with custom types) may not be fully mapped during import.
- Using `--pretty` can introduce significant whitespace in mixed-content elements.
