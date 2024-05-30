from dataclasses import dataclass, field

import pytest

from stencila_types import shortcuts as S
from stencila_types import types as T
from stencila_types.utilities import (
    STENCILA_CONVERTER,
    from_json,
    to_json,
)


@pytest.mark.skip_relaxed_json(
    lambda json_example: json_example.name() == "list",
)
def test_load_json_example(json_example):
    # Load
    node1 = from_json(json_example.path.read_text())

    # Round trip
    json_str = to_json(node1)
    node2 = from_json(json_str)

    # Check we're good
    assert node1 == node2


@dataclass(kw_only=True)
class Inner:
    size: int = 1
    shape: str = "cube"


@dataclass(kw_only=True)
class Wrapper:
    # Non stencila
    bob: str = "Hello"
    # Test we can handle unions
    some_nodes: list[T.Node]
    any_node: T.Node
    admonition: T.Admonition
    article: T.Article
    inner: Inner = field(default_factory=Inner)


def test_roundtrip_wrapped():
    c = STENCILA_CONVERTER

    adm = T.Admonition(admonition_type=T.AdmonitionType.Error, content=[S.p("bad!")])
    art = S.art(content=[S.h1("head"), S.p("Hello")])

    w1 = Wrapper(
        some_nodes=[S.p("Hello"), S.btn(name="Fred", code="push me")],
        any_node=S.td(S.p("hi")),
        admonition=adm,
        article=art,
    )
    d1 = c.unstructure(w1)
    w2 = c.structure(d1, Wrapper)
    assert w1 == w2
