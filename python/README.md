# Stencila SDK for Python

**Types and functions for using Stencila from within Python**

<a href="https://pypi.org/project/stencila/">
  <img src="https://img.shields.io/pypi/v/%40stencila%2Ftypes.svg?label=pypi%20stencila&color=1d3bd1&labelColor=3219a8">
</a>

## 👋 Introduction

This package provides Python classes for types in the [Stencila Schema](https://github.com/stencila/stencila/tree/main/schema#readme) and bindings to core [Stencila Rust](https://github.com/stencila/stencila/tree/main/rust#readme) functions.

The primary intended audience is developers who want to develop there own tools on top of Stencila's core functionality. For example, with this package you could construct Stencila documents programmatically using Python and write them to multiple formats (e.g. Markdown, JATS XML, PDF).

## 📦 Install

```console
python -m pip install stencila
```

## ⚡ Usage

### Types

The `types` module contains representations of all types in the Stencila Schema.

#### Object types

Object types (aka product types) in the Stencila Schema are represented as a `dataclass`. At present the `__init__` function requires keywords to be used (this is likely to be improved soon).

For example, to construct an article with a single "Hello world!" paragraph, you can construct `Article`, `Paragraph` and `Text`:

```py
from stencila.types import Article, Paragraph, Text

article = Article(content=[Paragraph(content=[Text(value="Hello world!")])]);

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

Use `from_string` to decode a string in a certain format to a schema type. Usually you will need to supply the `format` argument (it defaults to JSON). e.g.

```py
import asyncio

from stencila.convert import from_string

article = asyncio.run(
    from_string(
        '{type: "Article", content: [{type: "Paragraph", content: ["Hello world"]}]}',
        format="json5",
    )
)
```

#### `from_path`

Use `from_path` to decode a file system path, usually to a file, to a schema type. The format of the file can be supplied but if it is not is inferred from the file name. e.g.

```py
import asyncio

from stencila.convert import from_path

article = asyncio.run(
    from_path("my-article.jats.xml")
)
```

#### `to_string`

Use `to_string` to encode a schema type to a string. Usually you will want to supply the `format` argument (it defaults to JSON).

```py
import asyncio

from stencila.convert import to_string
from stencila.types import Article, Paragraph, Text

article = Article(content=[Paragraph(content=[Text(value="Hello world!")])])
markdown = asyncio.run(
    to_string(article, format="md")
)
```

#### `to_path`

To encode a schema type to a filesystem path, use `to_path`. e.g.

```py
import asyncio

from stencila.convert import to_path
from stencila.types import Article, Paragraph, Text

article = Article(content=[Paragraph(content=[Text(value="Hello world!")])])
asyncio.run(
    to_path(article, "my-article.md")
)
```

#### `from_to`

Use `from_to` when you want to convert a file to another format (i.e. as a more performance shortcut to combining `from_path` and `to_path`)

```py
import asyncio

from stencila.convert import from_to

asyncio.run(
    from_to("my-article.md", "my-article.html")
)
```

## 🛠️ Develop

### `types` module

Most of the types are generated from the Stencila Schema by the Rust [`schema-gen`](https://github.com/stencila/stencila/tree/main/rust/schema-gen#readme) crate. See there for contributing instructions.

### `convert` module

The `convert` module is implemented in Rust(`src/convert.rs`) with a thin Python wrapper (`python/stencila/convert.py`)to provide documentation and conversion to the types in the `types` module.

### Linting and testing

Please run linting checks and tests before contributing any code changes.

```console
make lint test
```
