# 📑 Schema

[![Build](https://travis-ci.org/stencila/schema.svg?branch=master)](https://travis-ci.org/stencila/schema)
[![Code coverage](https://codecov.io/gh/stencila/schema/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/schema)
[![Netlify](https://img.shields.io/netlify/b0e0d714-29f1-4ad1-8a7d-1af7799fb85b)](https://app.netlify.com/sites/stencila-schema/deploys)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://schema.stenci.la/)
[![Community](https://img.shields.io/badge/join-community-green.svg)](https://discord.gg/uFtQtk9)

## 👋 Introduction

This is the Stencila Schema, an extension to [schema.org](https://schema.org) to support structured, semantic, and executable documents. It also provides implementations of schema.org types (and our extensions) for several languages including JSON Schema, Typescript, Python and R. It is a central part of our platform that is used widely throughout our open-source tools as the data model for executable documents.

### Why an extension to schema.org?

Schema.org is _"a collaborative, community activity with a mission to create, maintain, and promote schemas for structured data on the Internet, on web pages, in email messages, and beyond."_ Schema.org is is used by most major search engines to provide richer, more semantic, search results. More and more web sites are using the schema.org vocabulary and there is increasing uptake in the research community e.g. bioschemas.org, codemeta.github.io

The [schema.org vocabulary](https://schema.org/docs/full.html) encompasses many varied concepts and topics. Of particular relevance to Stencila are types for research outputs such as [`ScholarlyArticle`](https://schema.org/CreativeWork), [`Dataset`](https://schema.org/Dataset) and [`SoftwareSourceCode`](https://schema.org/SoftwareSourceCode) and their associated meta data e.g. [`Person`](https://schema.org/Person), [`Organization`](https://schema.org/Organization), and [`Organization`](https://schema.org/Organization).

However, schema.org does not provide types for the _content_ of research articles. This is where our extensions come in. This schema adds types (and some properties to existing types) to be able to represent a complete executable, research article. These extensions types include "static" _nodes_ such as [`Paragraph`](https://schema.stenci.la/paragraph), [`Heading`](https://schema.stenci.la/heading) and [`Figure`](https://schema.stenci.la/figure), and "dynamic" nodes involved in execution such as [`CodeChunk`](https://schema.stenci.la/codechunk) and [`Parameter`](https://schema.stenci.la/parameter).

### It's about names, not formats

An important aspect of schema.org and similar vocabularies are that they really just define a shared way of naming things. They are format agnostic. As schema.org says, it can be used with _"many different encodings, including RDFa, Microdata and JSON-LD"_.

We extend this philosophy to the encoding of executable articles, allowing them to be encoded in several existing document formats. For example, the following very small [`Article`](https://schema.stenci.la/article), containing only one [`Paragraph`](https://schema.stenci.la/paragraph), and with no metadata, can be represented in Markdown:

```md
Hello world!
```

as YAML,

```yaml
type: Article
content:
  - type: Paragraph
    content:
      - Hello world!
```

as a Jupyter Notebook,

```json
{
  "nbformat": 4,
  "nbformat_minor": 4,
  "metadata": {
    "title": ""
  },
  "cells": [
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": [
        "Hello world!"
      ]
    }
  ]
}
```

as JSON-LD,

```json
{
  "@context": "http://schema.stenci.la/v1/jsonld/",
  "type": "Article",
  "content": [
    {
      "type": "Paragraph",
      "content": [
        "Hello world!"
      ]
    }
  ]
}
```

or as HTML with Microdata,

```html
<article itemscope="" itemtype="http://schema.org/Article">
  <p itemscope="" itemtype="http://schema.stenci.la/Paragraph">Hello world!</p>
</article>
```

This repository does not deal with format conversion per se. Please see [Encoda](https://github.com/stencila/encoda) for that. However, when developing our schema.org extensions, we aimed to not reinvent the wheel and maintain consistency and compatibility with existing _schemas_ for representing document content. Those include:

- [JATS XML](https://jats.nlm.nih.gov/)
- [MDAST](https://github.com/syntax-tree/mdast)
- [Open Document Format](http://docs.oasis-open.org/office/v1.2/OpenDocument-v1.2-part1.html)
- [Pandoc Types](https://github.com/jgm/pandoc-types)


### But, sometimes (often) we need more than just names

Despite its name, schema.org does not define strong rules around the _shape_ of data, as say a database schema or XML schema does. All the properties of schema.org types are optional, and although they have "expected types", this is not enforced. In addition, properties can be singular values or array,  but always have a singular name. For example, a `Article` has a `author` property which could be undefined, a string, a `Person` or an `Organization`, or an array of `Person` or `Organization` items.

This flexibility makes a lot of sense for the primary purpose of schema.org: semantic annotation of other content. However, for use as an internal data model, as in Stencila, it can result in a lot of defensive code to check exactly which of these alternatives a property value is. And writing more code than you need to is A Bad Thing™.

Instead, we wanted a schema that placed some restrictions on the shape of executable documents. This has flow on benefits for developer experience such as type inference and checking. To achieve this the Stencila Schema defines schema.org types using JSON Schema. Yes, that's a lot of "schemas", but bear with us...

### Using JSON Schema for validation and type safety

[JSON Schema](https://json-schema.org/) is _"a vocabulary that allows you to annotate and validate JSON documents"_. It is a draft internet standard, which like schema.org has a growing adoption e.g. [schemastore.org](https://www.schemastore.org/json/).

In Stencila Schema, when we define a type of document node, either a schema.org type, or an extension, we define it,

- as a JSON Schema document, with restrictions on the marginality, type and shape of it's properties
- using schema.org type and property names, pluralized as appropriate to avoid confusion

For example, an `Article` is defined to have an optional `authors` property (note the `s` this time) which is always an array whose items are either a `Person` or `Organization`.

```json
{
  "title": "Article",
  "@id": "schema:Article",
  "description": "An article, including news and scholarly articles.",
  "properties": {
    "authors": {
      "@id": "schema:author",
      "description": "The authors of this creative work.",
      "type": "array",
      "items": {
        "anyOf": [
          {
            "$ref": "Person.schema.json"
          },
          {
            "$ref": "Organization.schema.json"
          }
        ]
      }
    }
...

```

_To keep things simpler, this is a stripped down version of the actual[`Person.schema.json`](https://schema.stenci.la/Person.schema.json)._

With a JSON Schema, we are able to:

- use a JSON Schema validator to check that content meets the schema
- generate types (i.e. `interface` and `class` elements) matching the schema in other languages.

### But, JSON Schema can be a pain to write

JSON can be quite fiddly to write by hand. And JSON Schema lacks a way to easily express parent-child relationships between types. For these reasons, we define types using YAML with custom keywords such as `extends` and generate JSON Schema and ultimately bindings for each language from those.


## 📜 Documentation

Documentation is available at https://schema.stenci.la/.

Alternatively, you may want to directly consult the type definitions (`*.yaml` files) and documentation (`*.md` files) in the [`schema`](schema) directory.

## 🚀 Usage

### JSON-LD context

A JSON-LD `@context` is generated from the JSON Schema sources and published at https://schema.stenci.la/stencila.jsonld.

Individual files are published for each extension type e.g. https://schema.stenci.la/CodeChunk.jsonld and extension property e.g. https://schema.stenci.la/rowspan.jsonld


### Programming language bindings

Binding for this schema, in the form of installable packages, are currently generated for:

- [Python](https://stencila.github.io/schema/py/docs)
- [R](https://stencila.github.io/schema/r/docs)
- [Typescript](https://stencila.github.io/schema/ts/docs)

Depending on the capabilities of the host language, these packages expose type definitions as well as utility functions for constructing valid Stencila Schema nodes. Each packages has its own documentation auto-generated from the code.

## 🙏 Acknowledgments

Thanks to the developers of all the existing schemas and open source tools we use in this repo, including:

- Schema.org
- [CodeMeta](codemeta.github.io)
