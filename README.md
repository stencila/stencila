# Stencila schemas, protocols and other specs

[![Build](https://travis-ci.org/stencila/schema.svg?branch=master)](https://travis-ci.org/stencila/schema)
[![Code coverage](https://codecov.io/gh/stencila/schema/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/schema)
[![Netlify](https://img.shields.io/netlify/b0e0d714-29f1-4ad1-8a7d-1af7799fb85b)](https://app.netlify.com/sites/stencila-schema/deploys)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://schema.stenci.la/)
[![Community](https://img.shields.io/badge/join-community-green.svg)](https://discord.gg/uFtQtk9)

<!-- Automatically generated TOC. Don't edit, `make docs` instead -->

<!-- toc -->

- [Documentation](#documentation)
- [Language Specific Packages](#language-specific-packages)
- [JSON Schema definitions](#json-schema-definitions)
- [JSON-LD context](#json-ld-context)
- [Typescript type definitions](#typescript-type-definitions)

<!-- tocstop -->

This repository aims to document, and provide reference implementations for, the schemas, protocols and other
specifications used in Stencila.

As much as possible, we use existing specifications, and avoid defining any new ones. External specifications that we
currently use, or plan to use, include [Apache Avro], [JSON-LD], [JSON-RPC], [JSON-Schema], [Schema.org], [BioSchemas],
[CodeMeta] and [OpenSchemas]. In many ways, this repository simply documents how these existing standards are utilised
within Stencila.

## Documentation

Documentation is available at https://schema.stenci.la/.

## Language Specific Packages

Stencila Schema supports Python, R, and TypeScript with their respective packages.
Depending on the language capabilities, these packages expose type definitions as well as utility functions for
constructing valid Stencila Schema nodes.
Each packages has its own documentation auto-generated, and they can be found at:

- [Python](https://stencila.github.io/schema/py/docs)
- [R](https://stencila.github.io/schema/r/docs)
- [Typescript](https://stencila.github.io/schema/ts/docs)

## JSON Schema definitions

JSON Schemas are defined in the [`schema`](schema) directory.

## JSON-LD context

A JSON-LD `@context` is generated from the JSON Schema sources is available at https://schema.stenci.la/stencila.jsonld.

## Typescript type definitions

Typescript type definitions are generated from the JSON Schema sources and can be used by installing the Node.js package:

```bash
npm install @stencila/schema --save
```

[apache avro]: (https://avro.apache.org)
[bioschemas]: (https://bioschemas.org)
[codemeta]: (https://codemeta.github.io)
[json-ld]: (https://json-ld.org)
[json-rpc]: (https://www.jsonrpc.org)
[json-schema]: (https://json-schema.org)
[openschemas]: (https://openschemas.github.io)
[schema.org]: (https://schema.org)
