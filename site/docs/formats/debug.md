---
title: Debug
description: Rust debug representation of Stencila nodes
---
# Introduction

The Debug format is a plain-text representation of Stencila nodes using Rust's debug formatting. It is intended for inspection during development.

# Usage

Use the `.debug` file extension, or the `--to debug` option, when converting to Debug output e.g.

```sh
stencila convert doc.smd doc.debug
```

# Implementation

Debug output is implemented in the Rust crate [`codec-debug`](https://github.com/stencila/stencila/blob/main/rust/codec-debug).

# Notes

- Debug output is one-way and not intended for round-tripping.
- Use `--compact` for single-line output.
