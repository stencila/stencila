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

This library can also be used to interpret executable documents. Once installed it can be used like this:

```bash
$ python3 -m stencila.schema execute <inputfile> <outputfile> [parameters]
```

`inputfile` and/or `outputfile` can be set to `-` to read from stdin and/or write to stdout (respectively).

`[parameters]` is a list of parameters to pass to the document –- these will differ based on what the document defines.
They can be passed either by `--parameter_name=parameter_value` or `--parameter_name parameter_value`. Each parameter
must be named.

### Usage in Development

There are three options to run the interpreter without installing this package (which can be useful when developing).

#### setup.py develop

Run `python3 setup.py develop` which will link this library into your site packages directory. You can then execute
documents with the above Interpreter command.

#### Run interpreter.py directly

You can run the `interpreter.py` script directly, the arguments are the same as running as a module in the example
above except the first `execute` argument is omitted:

```bash
$ python3 schema/py/stencila/schema/interpreter.py <inputfile> <outputfile> [parameters]
```

### cd into stencila directory

You can run the interpreter as a module by changing into the `stencila` directory first, and then ommitting the
`stencila` namespace:

```bash
$ cd schema/py/stencila
$ python3 -m schema execute <inputfile> <outputfile> [parameters]
```
