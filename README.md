# Stencila schemas and other specifications

> ⚠ This repo contains pre-1.0 schemas and API specs which are draft and likely to change ⚠

[![Build status](https://travis-ci.org/stencila/schema.svg?branch=master)](https://travis-ci.org/stencila/schema)
[![Code coverage](https://codecov.io/gh/stencila/schema/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/schema)
[![Greenkeeper badge](https://badges.greenkeeper.io/stencila/schema.svg)](https://greenkeeper.io/)
[![NPM](http://img.shields.io/npm/v/@stencila/schema.svg?style=flat)](https://www.npmjs.com/package/@stencila/schema)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://stencila.github.io/schema/)
[![Chat](https://badges.gitter.im/stencila/stencila.svg)](https://gitter.im/stencila/stencila)

## TypeScript classes

Several types and properties defined in https://schema.org and https://codemeta.github.io are implemented in TypeScript. Currently, only types and properties required for other repos are implemented.

API documentation is available at https://stencila.github.io/schema/.

## JSON-LD context

A JSON-LD context is generated from the TypeScript sources and is available at https://stencila.github.io/schema/context.jsonld

A draft JSON Schema for `Cells` is defined in [`src/Cell.yaml`](src/Cell.yaml). This will be ported to the JSON-LD context in the near future.

## `Host` API

A draft [OpenAPI specification](https://github.com/OAI/OpenAPI-Specification) for `Hosts` is defined in [`src/Host.yaml`](src/Host.yaml) and is available as more reader-friendly, browserable HTML [here](https://stencila.github.io/schema/host.html).

This API is implemented (to varying degress) in the following packages:

- [stencila/py](https://github.com/stencila/py)
- [stencila/r](https://github.com/stencila/r)
- [stencila/js](https://github.com/stencila/js)
- [stencila/cloud](https://github.com/stencila/cloud)
