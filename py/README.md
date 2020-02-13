# Stencila Schema for Python

[![Build Status](https://travis-ci.org/stencila/schema.svg?branch=master)](https://travis-ci.org/stencila/schema)
[![Code coverage](https://badger.nokome.now.sh/codecov-folder/stencila/schema/py)](https://codecov.io/gh/stencila/schema/tree/master/py)
[![Code style](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)
[![PyPI](https://img.shields.io/pypi/v/stencila-schema.svg)](https://pypi.org/project/stencila-schema)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://stencila.github.io/schema/py/docs)

This package provides Python bindings for the [Stencila Schema](https://schema.stenci.la).
It is primarily aimed at Python developers wanting to programmatically generate, or modify, executable documents. For example, it is used in [`pyla`](https://github.com/stencila/pyla), an interpreter for executable documents containing Python code.

## Install

```python
pip3 install stencila-schema
```

## Use

This packages exports a constructor function for each type of document node in the Stencila Schema e.g. `Article`, `Paragraph`, `CodeChunk`. To support conversion of schema nodes to/from JSON `json.py` defines `encode` and `decode` functions. e.g.

```python
from stencila.schema.types import Heading
from stencila.schema.json import encode, decode

heading = Heading(["Heading Text"], 2)

json = encode(heading)
#{\n  "type": "Heading",\n  "content": [\n    "Heading Text"\n  ],\n  "depth": 2\n}

decode(json)
# <stencila.schema.types.Heading object at 0x7fda7bbdd780>
```
