<!--
SPDX-FileCopyrightText: 2024 Nokome Bentley

SPDX-License-Identifier: Apache-2.0
-->

# Stencila SDK for Python

**Bindings for using Stencila from Python**

<a href="https://pypi.org/project/stencila/">
    <img src="https://img.shields.io/pypi/v/stencila.svg?logo=python&label=stencila&style=for-the-badge&color=1d3bd1&logoColor=66ff66&labelColor=3219a8">
</a>

## 👋 Introduction

This package provides Python classes for types in the [Stencila Schema](https://github.com/stencila/stencila/tree/main/schema#readme) and bindings to core [Stencila Rust](https://github.com/stencila/stencila/tree/main/rust#readme) functions.

The primary intended audience is developers who want to develop their own tools on top of Stencila's core functionality. For example, with this package you could construct Stencila documents programmatically using Python and write them to multiple formats (e.g. Markdown, JATS XML, PDF).

> [!IMPORTANT]
> At present, there are only bindings to functions for format conversion, but future versions will expand this scope to include document management (e.g branching and merging) and execution.

## 📦 Install

```console
python -m pip install stencila
```

> [!NOTE]
> If you encounter problems with the above command, you may need to upgrade Pip using `pip install --upgrade pip`.
>
> This is due to a [change in the dependency resolver](https://pip.pypa.io/en/latest/user_guide/#changes-to-the-pip-dependency-resolver-in-20-3-2020) in Pip 20.3.

## ⚡ Usage

### Types

The `types` module contains representations of all types in the Stencila Schema.

#### Object types

Object types (aka product types) in the Stencila Schema are represented as a `dataclass`. At present the `__init__` function requires keywords to be used (this is likely to be improved soon).

For example, to construct an article with a single "Hello world!" paragraph, you can construct `Article`, `Paragraph` and `Text`:

```py
from stencila.types import Article, CreativeWork, Paragraph, Text, Thing

article = Article(content=[Paragraph(content=[Text(value="Hello world!")])])

assert isinstance(article, Article)
assert isinstance(article, CreativeWork)
assert isinstance(article, Thing)

assert isinstance(article.content[0], Paragraph)

assert isinstance(article.content[0].content[0], Text)
```

#### Union types

Union types (aka sum types) in the Stencila Schema are represented as `typing.Union`. For example, the `Block` union type is defined like so:

```py
Block = Union[
    Call,
    Claim,
    CodeBlock,
    CodeChunk,
    Division,
    Figure,
    For,
    Form,
    Heading,
...
```

#### Enumeration types

Enumeration types in the Stencila Schema are represented as `StrEnum`. For example, the `CitationIntent` enumeration is defined like so:

```py
class CitationIntent(StrEnum):
    """
    The type or nature of a citation, both factually and rhetorically.
    """

    AgreesWith = "AgreesWith"
    CitesAsAuthority = "CitesAsAuthority"
    CitesAsDataSource = "CitesAsDataSource"
    CitesAsEvidence = "CitesAsEvidence"
    CitesAsMetadataDocument = "CitesAsMetadataDocument"
    CitesAsPotentialSolution = "CitesAsPotentialSolution"
    CitesAsRecommendedReading = "CitesAsRecommendedReading"
    CitesAsRelated = "CitesAsRelated"
```

### Conversion

The `convert` module has five functions for encoding and decoding Stencila documents and for converting documents between formats. All functions are `async`.

#### `from_string`

Use `from_string` to decode a string in a certain format to a Stencila Schema type. Usually you will need to supply the `format` argument (it defaults to JSON). e.g.

```py
import asyncio

from stencila.convert import from_string

doc = asyncio.run(
    from_string(
        '''{
            type: "Article",
            content: [{
                type: "Paragraph",
                content: [{
                    type: "Text",
                    value: "Hello world"
                }]
            }]
        }''',
        format="json5",
    )
)
```

#### `from_path`

Use `from_path` to decode a file system path (usually a file) to a Stencila Schema type. The format can be supplied but if it is not is inferred from the path. e.g.

```py
import asyncio

from stencila.convert import from_path

doc = asyncio.run(from_path("doc.jats.xml"))
```

#### `to_string`

Use `to_string` to encode a Stencila Schema type to a string. Usually you will want to supply the `format` argument (it defaults to JSON).

```py
import asyncio

from stencila.convert import to_string
from stencila.types import Article, Paragraph, Text

doc = Article([Paragraph([Text("Hello world!")])])

markdown = asyncio.run(to_string(doc, format="md"))
```

#### `to_path`

To encode a Stencila Schema type to a filesystem path, use `to_path`. e.g.

```py
import asyncio

from stencila.convert import to_path
from stencila.types import Article, Paragraph, Text

doc = Article([Paragraph([Text("Hello world!")])])

asyncio.run(to_path(doc, "doc.md"))
```

#### `from_to`

Use `from_to` when you want to convert a file to another format (i.e. as a more performant shortcut to combining `from_path` and `to_path`)

```py
import asyncio

from stencila.convert import from_to

asyncio.run(from_to("doc.md", "doc.html"))
```

> [!NOTE]
> Some of the usage examples above illustrate manually constructing in-memory Python representations of small documents. This is for illustration only and would be unwieldy for large documents. Instead we imagine developers using the `convert.from_string` or `convert.from_path` functions to load documents into memory from other formats, or writing functions to construct documents composed of the Stencila classes.

## 🛠️ Develop

### Bindings

This packages uses [PyO3](https://pyo3.rs) and [Maturin](https://maturin.rs) to generate a Python native extension from Stencila Rust functions. It uses the [layout](https://www.maturin.rs/project_layout#mixed-rustpython-project) recommended for mixed Rust/Python projects: Rust code is in `src` and Python code and tests is in `python`.

To build the native extension and use it in a Python shell:

```console
make run
```

To build the native extension for the current platform (for several versions of Python):

```console
make build
```

### Linting and testing

Please run linting and tests before contributing any code changes.

```console
make lint test
```

There is also a `make fix` recipe that will fix any formatting or linting issues.

### Testing on different Python versions

You can use `asdf` to test this package across different versions of Python:

```console
asdf install python 3.9.18
asdf local python 3.9.18
poetry env use 3.9.18
poetry install
make test
```

> [!NOTE]
> In the future, we may use `tox` (or similar) to run tests across Python versions. But how to make that work with `pyo3` and `maturin` is yet to be resolved.

### Code organization

#### `types` module

Most of the types are generated from the Stencila Schema by the Rust [`schema-gen`](https://github.com/stencila/stencila/tree/main/rust/schema-gen#readme) crate. See there for contributing instructions.

#### `convert` module

The `convert` module is implemented in Rust (`src/convert.rs`) with a thin Python wrapper (`python/stencila/convert.py`) to provide documentation and conversion to the types in the `types` module.
