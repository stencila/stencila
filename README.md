# 📑 Schema

**Extensions to schema.org to support semantic, composable, parameterize-able and executable documents**

<br>

[![Build Status](https://dev.azure.com/stencila/stencila/_apis/build/status/stencila.schema?branchName=master)](https://dev.azure.com/stencila/stencila/_build/latest?definitionId=9&branchName=master)
[![Code coverage](https://codecov.io/gh/stencila/schema/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/schema)
[![Netlify](https://img.shields.io/netlify/b0e0d714-29f1-4ad1-8a7d-1af7799fb85b)](https://app.netlify.com/sites/stencila-schema/deploys)
[![Community](https://img.shields.io/badge/join-community-green.svg)](https://discord.gg/uFtQtk9)


|  |||
|--|-------|-------------|
JSON-LD | [![Context](https://img.shields.io/badge/json--ld-%40context-success)](https://schema.stenci.la/stencila.jsonld) | [![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://schema.stenci.la/)
JSON Schema | [![Context](https://img.shields.io/badge/json%20schema-v1-success)](https://unpkg.com/browse/@stencila/schema@1/dist/) | [![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://schema.stenci.la/)
TypeScript/JavaScript | [![NPM](https://img.shields.io/npm/v/@stencila/schema.svg?style=flat)](https://www.npmjs.com/package/@stencila/schema) | [![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://stencila.github.io/schema/ts/docs) |
Python | [![PyPI](https://img.shields.io/pypi/v/stencila-schema.svg)](https://pypi.org/project/stencila-schema) | [![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://stencila.github.io/schema/py/docs) |
R | [![CRAN](https://www.r-pkg.org/badges/version/stencilaschema)](https://cran.r-project.org/web/packages/stencilaschema/) | [![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://stencila.github.io/schema/r/docs) |

<br>

## 👋 Introduction

This is the Stencila Schema, an extension to [schema.org](https://schema.org) to support semantic, composable, parameterize-able and executable documents (we call them _stencils_ for short). It also provides implementations of schema.org types (and our extensions) for several languages including JSON Schema, Typescript, Python and R. It is a central part of our platform that is used widely throughout our open-source tools as the data model for executable documents.

### Why an extension to schema.org?

Schema.org is _"a collaborative, community activity with a mission to create, maintain, and promote schemas for structured data on the Internet, on web pages, in email messages, and beyond."_. Schema.org is is used by most major search engines to provide richer, more semantic, search results. More and more web sites are using the schema.org vocabulary and there is increasing uptake in the research community e.g. bioschemas.org, codemeta.github.io

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
      "source": ["Hello world!"]
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
      "content": ["Hello world!"]
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

Despite its name, schema.org does not define strong rules around the _shape_ of data, as say a database schema or XML schema does. All the properties of schema.org types are optional, and although they have "expected types", this is not enforced. In addition, properties can be singular values or array, but always have a singular name. For example, a `Article` has a `author` property which could be undefined, a string, a `Person` or an `Organization`, or an array of `Person` or `Organization` items.

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

## 🛠 Contributing

We 💕 contributions! All contributions: ideas 🤔, examples 💡, bug reports 🐛, documentation 📖, code 💻, questions 💬.

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for a guide on how to contribute to the schema definitions. See the `README.md` files of each language sub-folder e.g. [`py`](py) for advice on development of language bindings and [issue](https://github.com/stencila/schema/issues/256) for how to add you or others to the following _important_ table:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tr>
    <td align="center"><a href="http://has100ideas.com"><img src="https://avatars0.githubusercontent.com/u/57006?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Mac Cowell</b></sub></a><br /><a href="https://github.com/stencila/schema/commits?author=100ideas" title="Code">💻</a> <a href="#ideas-100ideas" title="Ideas, Planning, & Feedback">🤔</a></td>
    <td align="center"><a href="http://toki.io"><img src="https://avatars1.githubusercontent.com/u/10161095?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Jacqueline</b></sub></a><br /><a href="https://github.com/stencila/schema/commits?author=jwijay" title="Code">💻</a> <a href="https://github.com/stencila/schema/commits?author=jwijay" title="Documentation">📖</a> <a href="#ideas-jwijay" title="Ideas, Planning, & Feedback">🤔</a></td>
    <td align="center"><a href="https://github.com/beneboy"><img src="https://avatars1.githubusercontent.com/u/292725?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Ben Shaw</b></sub></a><br /><a href="https://github.com/stencila/schema/commits?author=beneboy" title="Code">💻</a> <a href="#ideas-beneboy" title="Ideas, Planning, & Feedback">🤔</a> <a href="#infra-beneboy" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="https://github.com/stencila/schema/commits?author=beneboy" title="Documentation">📖</a></td>
    <td align="center"><a href="http://ketch.me"><img src="https://avatars2.githubusercontent.com/u/1646307?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Alex Ketch</b></sub></a><br /><a href="https://github.com/stencila/schema/commits?author=alex-ketch" title="Code">💻</a> <a href="https://github.com/stencila/schema/commits?author=alex-ketch" title="Documentation">📖</a> <a href="#design-alex-ketch" title="Design">🎨</a></td>
    <td align="center"><a href="https://github.com/nokome"><img src="https://avatars0.githubusercontent.com/u/1152336?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Nokome Bentley</b></sub></a><br /><a href="https://github.com/stencila/schema/commits?author=nokome" title="Code">💻</a> <a href="https://github.com/stencila/schema/commits?author=nokome" title="Documentation">📖</a> <a href="#ideas-nokome" title="Ideas, Planning, & Feedback">🤔</a></td>
    <td align="center"><a href="https://github.com/asisiuc"><img src="https://avatars0.githubusercontent.com/u/17000527?v=4?s=100" width="100px;" alt=""/><br /><sub><b>asisiuc</b></sub></a><br /><a href="https://github.com/stencila/schema/commits?author=asisiuc" title="Code">💻</a> <a href="#ideas-asisiuc" title="Ideas, Planning, & Feedback">🤔</a></td>
    <td align="center"><a href="https://github.com/apawlik"><img src="https://avatars2.githubusercontent.com/u/2358535?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Aleksandra Pawlik</b></sub></a><br /><a href="https://github.com/stencila/schema/commits?author=apawlik" title="Code">💻</a> <a href="https://github.com/stencila/schema/commits?author=apawlik" title="Documentation">📖</a> <a href="#ideas-apawlik" title="Ideas, Planning, & Feedback">🤔</a></td>
  </tr>
  <tr>
    <td align="center"><a href="https://vsoch.github.io"><img src="https://avatars0.githubusercontent.com/u/814322?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Vanessasaurus</b></sub></a><br /><a href="#ideas-vsoch" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/stencila/schema/commits?author=vsoch" title="Code">💻</a></td>
    <td align="center"><a href="https://github.com/rgieseke"><img src="https://avatars.githubusercontent.com/u/198537?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Robert Gieseke</b></sub></a><br /><a href="#ideas-rgieseke" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/stencila/schema/commits?author=rgieseke" title="Code">💻</a> <a href="https://github.com/stencila/schema/commits?author=rgieseke" title="Documentation">📖</a></td>
  </tr>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## 🙏 Acknowledgments

Thanks to the developers of the existing schemas and open source tools we use, or have been inspired by, including:

- [BioSchemas](https://bioschemas.org/)
- [CiTO](http://www.sparontologies.net/ontologies/cito)
- [CodeMeta](https://codemeta.github.io)
- [JSON Schema](https://json-schema.org/)
- [Schema.org](https://schema.org)
