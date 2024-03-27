# Stencila Types for Python

[![stencila_types](https://img.shields.io/pypi/v/stencila_types.svg?logo=python&label=stencila_types&style=for-the-badge&color=1d3bd1&logoColor=66ff66&labelColor=3219a8)](https://pypi.org/project/stencila_types/)

## Introduction

This package provides Python classes for types in the [Stencila Schema](https://github.com/stencila/stencila/tree/main/schema#readme), shortcuts for easily constructing these types, and utilities for loading and saving the types to JSON.

## ⚡ Usage

### Object types

Object types (aka product types) in the Stencila Schema are represented as a `dataclass`.
For example, to construct an article with a single "Hello world!" paragraph, you can construct `Article`, `Paragraph` and `Text`:

```py
from stencila_types.types import Article, CreativeWork, Paragraph, Text, Thing

article = Article(content=[Paragraph(content=[Text(value="Hello world!")])])

assert isinstance(article, Article)
assert isinstance(article, CreativeWork)
assert isinstance(article, Thing)

assert isinstance(article.content[0], Paragraph)

assert isinstance(article.content[0].content[0], Text)
```

### Union types

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

### Enumeration types

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

### Shortcuts

Constructing complex Stencila types can be more easily constructed using the shortcuts module.

```py
from stencila_types import types as T
from stencila_types import shortcuts as S

# As above
art1 = T.Article(content=[T.Paragraph(content=[T.Text(value="Hello world!")])])

# Using shortcuts
art2 = S.art(S.p("Hello world!"))

assert art1 == art2
```

### Basic JSON support

```py
import json
from stencila_types.utilities import from_json, to_json

# Using shortcuts
art1 = S.art(S.p("Hello world!"))

s = to_json(art1)
assert s.startswith('{"type": "Article", "id": null,')
art2 = from_json(s)

assert art1 == art2
```
