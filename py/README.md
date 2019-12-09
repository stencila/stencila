# Stencila Schema Bindings for Python

This is the Python implementation of Schema types for [Stencila Schema](https://stencila.github.io/schema/).

## Types

Types are defined in `types.py`.

## Utilities

To support conversion of Stencila types to/from JSON (via `dict`), `util.py` defines `to_dict`, `from_dict`,
`to_json` and `from_json` functions.

## Example Python to JSON

```python
from stencila.schema.types import Heading
from stencila.schema.util import to_json

h2 = Heading(["Heading Text"], 2)

serialized = to_json(h2)
# {"type": "heading", "content": ["Heading Text"], "depth": 2}
```

## Example JSON to Python

```python
from stencila.schema.util import from_json

serialized = """{"type": "Heading", "content": ["Heading Text"], "depth": 2}"""

h2 = from_json(serialized)
```

## Interpreter

Executing/interpreting executable documents has been moved to its own project,
[Pyla](https://github.com/stencila/pyla).
