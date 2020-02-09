# Stencila Schema for Python

This package provides Python bindings for [Stencila Schema](https://stencila.github.io/schema/).

## Install

```r
pip3 install stencila-schema
```

## Use

This package is primarily aimed at Python developers wanting to programmatically generate, or modify, executable documents. It exports a constructor function for each type of document node in the Stencila Schema e.g. `Article`, `Paragraph`, `CodeChunk`.

## Utilities

To support conversion of Stencila types to/from JSON (via `dict`), `util.py` defines `to_dict`, `from_dict`,
`to_json` and `from_json` functions.

### Example Python to JSON

```python
from stencila.schema.types import Heading
from stencila.schema.util import to_json

h2 = Heading(["Heading Text"], 2)

serialized = to_json(h2)
# {"type": "heading", "content": ["Heading Text"], "depth": 2}
```

### Example JSON to Python

```python
from stencila.schema.util import from_json

serialized = """{"type": "Heading", "content": ["Heading Text"], "depth": 2}"""

h2 = from_json(serialized)
```
