# Stencila Schema for Python

This package provides Python bindings for the [Stencila Schema](https://schema.stenci.la).
It is primarily aimed at Python developers wanting to programmatically generate, or modify, executable documents. For example, it is used in [`pyla`](https://github.com/stencila/pyla), an interpreter for executable documents containing Python code.

## Install

```r
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
