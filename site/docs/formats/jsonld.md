---
title: JSON-LD
description: JavaScript Object Notation for Linked Data
---
# Introduction

[JSON-LD](https://json-ld.org/), or JSON for Linked Data, is a lightweight data interchange format designed to express linked data in a format that is both human-readable and machine-friendly. It extends JSON by providing a standard way to embed linked data within JSON documents, allowing for a network of standards-based, machine-readable, structured data on the web.

Stencila provides support for JSON-LD for storing and transferring documents in a format with high interoperability.

# Usage

Use the `.jsonld` file extension, or the `--to jsonld` or `--from jsonld` options, when converting to/from JSON-LD e.g.

```sh
stencila convert doc.smd doc.jsonld
```

By default, the encoded JSON-LD is indented. The `--compact` option can be used to produce un-indented, single line JSON-LD.

# Implementation

Stencila Schema is based on [schema.org](https://schema.org) and has a JSON-LD `@context` published at https://stencila.org/context.jsonld. When Stencila documents are exported as JSON, this context is applied. As such, the JSON documents that Stencila produces are inherently JSON-LD documents.

For example, an `Article` is exported like so:

```json
{
  "$schema": "https://stencila.org/Article.schema.json",
  "@context": "https://stencila.org/context.jsonld",
  "type": "Article",
  "content": [
    {
      "type": "Paragraph",
      "content": [
        {
          "type": "Text",
```

However, because the the schema.org is the most widely used vocabulary for JSON-LD, the `JsonLdCodec` translates terms in the Stencila context, to those in the schema.org context, and uses schema.org as the [default vocabulary](https://www.w3.org/TR/json-ld11/#default-vocabulary), with the Stencila context as an extension. This saves consumers of the JSON-LD from having to do this translation themselves.

In addition, when exporting to JSON-LD, the `@type` and `@id` [keywords](https://www.w3.org/TR/json-ld11/#syntax-tokens-and-keywords) are used instead of `type` and `id`.

For example, the above article as exported to JSON-LD as follows. Note that because the types `Article` and `Text` are part of schema.org, there is no need to prefix their name. However because schema.org does not have a `Paragraph` type or a `content` property, it is necessary to prefix those with `stencila:`.

```json
{
  "@context": {
    "@vocab": "https://schema.org/",
    "stencila": "https://stencila.org/"
  },
  "@type": "Article",
  "stencila:content": [
    {
      "@type": "stencila:Paragraph",
      "stencila:content": [
        {
          "@type": "Text",
```

# Notes

- JSON-LD output uses schema.org as the default vocabulary with Stencila as an extension.
- The `--compact` option produces single-line JSON-LD output.
