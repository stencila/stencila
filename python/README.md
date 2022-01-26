# Stencila Schema for Python

[![Build Status](https://dev.azure.com/stencila/stencila/_apis/build/status/stencila.schema?branchName=master)](https://dev.azure.com/stencila/stencila/_build/latest?definitionId=9&branchName=master)
[![Code coverage](https://badger.nokome.now.sh/codecov-folder/stencila/schema/python)](https://codecov.io/gh/stencila/schema/tree/master/python)
[![Code style](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)
[![PyPI](https://img.shields.io/pypi/v/stencila-schema.svg)](https://pypi.org/project/stencila-schema)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://stencila.github.io/schema/python/docs)

This package provides Python bindings for the [Stencila Schema](https://schema.stenci.la) for executable documents.
It is primarily aimed at Python developers wanting to programmatically generate, or modify, executable documents. For example, it is used in [`pyla`](https://github.com/stencila/pyla), a Stencila plugin for Python.

## Install

```python
pip3 install stencila-schema
```

## Use

This packages exports a Python class for each type of document node in the Stencila Schema e.g. `Article`, `Paragraph`, `CodeChunk`.

For type safety, type annotations are places on attributes and parameters of the `__init__` method. e.g.

```python
class CodeExpression(CodeFragment):
    """An expression defined in programming language source code."""

    errors: Optional[Array["CodeError"]] = None
    """Errors when compiling or executing the chunk."""

    output: Optional["Node"] = None
    """The value of the expression when it was last evaluated."""
```

The `__init__` method of each class has as parameters the attributes of the class (including those that are inherited) with required attributes first (alphabetically where there are more than one), followed by optional attributes (also alphabetically) e.g. for `CodeExpression`:

```python
    def __init__(
        self,
        text: str,
        errors: Optional[Array["CodeError"]] = None,
        format: Optional[str] = None,
        id: Optional[str] = None,
        meta: Optional[Dict[str, Any]] = None,
        output: Optional["Node"] = None,
        programmingLanguage: Optional[str] = None
    )
```

It is recommended to use keyword arguments when calling constructors as it substantially reduces the likelihood that your code will break if you get the order wrong or if there are changes in the attributes of classes (and thus their order in `__init__` parameters) in later versions e.g.

```python
from stencila.schema.types import Article, CodeExpression, Paragraph, Person

article = Article(
    title="My first executable document",
    authors=[Person(givenNames=["Jane"], familyNames=["Doe"])],
    content=[
        Paragraph(
            content=[
                "Two times two: ",
                CodeExpression(programmingLanguage="python", text="2 * 2"),
            ]
        )
    ],
)

print(article.authors[0].givenNames)
# Jane
```

In contrast, the following code is more concise, but is broken because, although it provides all required arguments, it gets the order wrong:

```python
from stencila.schema.types import Article, CodeExpression, Paragraph, Person

article = Article(
    "My first executable document",
    [Person(["Jane"], ["Doe"])],
    [Paragraph(["Two times two: ", CodeExpression("2 * 2", "python"),])],
)

print(article.authors[0].address)
# Jane

print(article.authors[0].givenNames)
# None
```

To support conversion of schema nodes to/from JSON, `json.py` defines `encode` and `decode` functions. e.g.

```python
from stencila.schema.types import Heading
from stencila.schema.json import encode, decode

heading = Heading(content=["Heading Text"], depth=2)
#<stencila.schema.types.Heading object at 0x7f2d038a3748>

json = encode(heading)
print(json)
#{
#  "type": "Heading",
#  "content": [
#    "Heading Text"
#  ],
#  "depth": 2
#}

decode(json)
#<stencila.schema.types.Heading object at 0x7fda7bbdd780>
```
