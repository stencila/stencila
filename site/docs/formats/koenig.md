---
title: Koenig JSON
description: Serialization format for the Ghost CMS editor
---

# Introduction

[Koenig](https://github.com/TryGhost/Koenig) is the editor developed by and used within the [Ghost](https://ghost.org) content management system. Koenig is built on the [Lexical](https://lexical.dev) editor framework and extends [Lexical JSON](../lexical) with additional card types for Ghost-specific content such as image galleries, embedded content, and email CTAs.

Stencila supports conversion to/from Koenig JSON to enable publishing Stencila documents to Ghost.

# Usage

Use the `.koenig` file extension, or the `--to koenig` or `--from koenig` options, when converting to/from Koenig JSON e.g.

```sh
stencila convert doc.smd doc.koenig
```

# Implementation

Stencila supports bi-directional conversion between Stencila documents and Koenig JSON, implemented in the Rust crate [`codec-lexical`](https://github.com/stencila/stencila/blob/main/rust/codec-lexical). The codec builds on the Lexical JSON codec with additional transformer functions for Koenig-specific card types.

# Limitations

- Stencila node types that have no Lexical or Koenig card representation are lost during export.
- Ghost-specific card types (e.g. product cards, signup forms, email content cards) may not fully map to Stencila nodes during import.
- Koenig's content model is simpler than Stencila's; complex document structures are flattened.
