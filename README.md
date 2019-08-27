# Stencila schemas, protocols and other specs

![Status](https://img.shields.io/badge/status-draft-orange.svg)
[![Build](https://travis-ci.org/stencila/schema.svg?branch=master)](https://travis-ci.org/stencila/schema)
[![Netlify Status](https://api.netlify.com/api/v1/badges/b0e0d714-29f1-4ad1-8a7d-1af7799fb85b/deploy-status)](https://app.netlify.com/sites/stencila-schema/deploys)
[![Code coverage](https://codecov.io/gh/stencila/schema/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/schema)
[![NPM](http://img.shields.io/npm/v/@stencila/schema.svg?style=flat)](https://www.npmjs.com/package/@stencila/schema)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://stencila.github.io/schema/)
[![Community](https://img.shields.io/badge/join-community-green.svg)](https://community.stenci.la)
[![Chat](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila)

> :sparkles:
> This is a **work in progress**. But comments, suggestions, and pull requests are very much appreciated
> :sparkles:

<!-- Automatically generated TOC. Don't edit, `make docs` instead>

<!-- toc -->

- [Documentation](#documentation)
- [JSON Schema definitions](#json-schema-definitions)
- [JSON-LD context](#json-ld-context)
- [Typescript type definitions](#typescript-type-definitions)

<!-- tocstop -->

This repository aims to document, and provide reference implementations for, the schemas, protocols and other specifications used in Stencila.

As much as possible, we use existing specifications, and avoid defining any new ones. External specifications that we currently use, or plan to use, include [Apache Avro], [JSON-LD], [JSON-RPC], [JSON-Schema], [Schema.org], [BioSchemas], [CodeMeta] and [OpenSchemas]. In many ways, this repository simply documents how these existing standards are utilised within Stencila.

## Documentation

Documentation is available at https://schema.stenci.la/.

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
