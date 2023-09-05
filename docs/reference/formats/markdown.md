# Markdown

## Introduction

Markdown is a lightweight markup language widely used for formatting plain text documents. It provides a simple and human-readable way to structure text and add basic styling, such as headers, lists, links, and emphasis. Markdown's benefits include ease of use, and compatibility with various web and documentation platforms.

## Implementation

Stencila support bi-directional conversion between Stencila documents and Markdown. 

### Stencila to Markdown

The `codec-markdown-trait` Rust crate defines the `MarkdownCodec` crate which has the `to_markdown` method. The `codec-markdown-derive` crate provides a derive macro which is used to derive `MarkdownCodec` for all types in the Stencila Schema.
