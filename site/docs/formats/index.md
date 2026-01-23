---
title: Formats
description: Formats supported by Stencila codecs
---
# Introduction

Stencila supports a range of document and data formats via codecs. Each codec converts to and from the Stencila schema, enabling bidirectional conversion between formats where supported.

# Codecs architecture

Codecs encode external formats into Stencila's schema nodes and decode them back again. This mirrors Pandoc's approach: conversion goes through a canonical internal representation so any format with both directions can convert between each other. Some codecs are fully lossless (for example, JSON, CBOR, YAML), while others are lossy because the source format lacks structure for all Stencila nodes.

# Testing and examples

Stencila uses property testing to exercise codec round-trips and detect regressions; see `rust/codecs/tests/proptests.rs` for the property test suite. The `examples/conversion` folder contains snapshots of Stencila node types serialized into various formats, which are used for regression testing and documentation.

# Formats

| Format | Default extension | Docs |
| --- | --- | --- |
| CBOR | `cbor` | [CBOR](cbor) |
| CBOR+Zstd | `czst` | [CBOR+Zstd](cborzstd) |
| CSL-JSON | `csl` | [CSL-JSON](csl) |
| CSV | `csv` | [CSV](csv) |
| Citation File Format | `cff` | [Citation File Format](cff) |
| DOM HTML | `dom.html` | [DOM HTML](dom) |
| Debug | `debug` | [Debug](debug) |
| Email HTML | `email.html` | [Email HTML](email) |
| HTML | `html` | [HTML](html) |
| IPYNB | `ipynb` | [IPYNB](ipynb) |
| JATS | `jats.xml` | [JATS](jats) |
| JSON | `json` | [JSON](json) |
| JSON+Zip | `json.zip` | [JSON+Zip](jsonzip) |
| JSON-LD | `jsonld` | [JSON-LD](jsonld) |
| JSON5 | `json5` | [JSON5](json5) |
| Koenig JSON | `koenig` | [Koenig JSON](koenig) |
| LLM Markdown | `llmd` | [LLM Markdown](llmd) |
| LaTeX | `latex` | [LaTeX](latex) |
| Lexical JSON | `lexical` | [Lexical JSON](lexical) |
| MJML | `mjml` | [MJML](mjml) |
| Markdown | `md` | [Markdown](md) |
| Meca | `meca` | [Meca](meca) |
| Microsoft Excel | `xlsx` | [Microsoft Excel](xlsx) |
| Microsoft Excel (XLS) | `xls` | [Microsoft Excel (XLS)](xls) |
| Microsoft Word | `docx` | [Microsoft Word](docx) |
| MyST Markdown | `myst` | [MyST Markdown](myst) |
| OpenDocument Spreadsheet | `ods` | [OpenDocument Spreadsheet](ods) |
| OpenDocument Text | `odt` | [OpenDocument Text](odt) |
| PDF | `pdf` | [PDF](pdf) |
| PNG | `png` | [PNG](png) |
| Pandoc AST | `pandoc` | [Pandoc AST](pandoc) |
| Plain text | `text` | [Plain text](text) |
| PubMed Central OA Package | `pmcoa` | [PubMed Central OA Package](pmcoa) |
| Quarto Markdown | `qmd` | [Quarto Markdown](qmd) |
| R+LaTeX | `rnw` | [R+LaTeX](rnw) |
| Stencila Markdown | `smd` | [Stencila Markdown](smd) |
| TSV | `tsv` | [TSV](tsv) |
| TeX | `tex` | [TeX](tex) |
| YAML | `yaml` | [YAML](yaml) |

Deprecated formats `directory` and `swb` are not documented.
