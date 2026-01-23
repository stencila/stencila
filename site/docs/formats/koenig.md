---
title: Koenig JSON
description: Format for integrating with the Koenig editor
---
# Introduction

[Koenig](https://github.com/TryGhost/Koenig) is an editor developed by, and used within, the [Ghost](https://ghost.org) content management system. Koenig is based on the [Lexical](https://lexical.dev) editor framework and this format simply extends [Lexical JSON](../lexical) to accommodate some of the customizations of the Koenig document schema.

Stencila supports conversion to/from Koenig JSON to support publishing Stencila documents to Ghost.

# Usage

Use the `.koenig` file extension, or the `--to koenig` or `--from koenig` options, when converting to/from Koenig JSON e.g.

```sh
stencila convert doc.smd doc.koenig
```

# Implementation

Stencila supports bi-directional conversion between Stencila documents and Koenig JSON. This is built on top of [`serde_json`](https://crates.io/crates/serde_json) with transformer functions to map between Lexical/Koenig node types and Stencila node types.

# Notes

- Koenig JSON targets Ghost publishing workflows.
- Conversion is lossy for features that have no Lexical/Koenig representation.
