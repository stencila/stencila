from dataclasses import dataclass

import pytest
from beartype.roar import BeartypeCallHintParamViolation
from cattrs import unstructure

from stencila_types import shortcuts as S
from stencila_types import types as T


@dataclass
class NotAnInline:
    content: str


# Start testing -----------------
def test_emphasis(data_regression):
    t = S.em("Banana")
    data_regression.check(unstructure(t))

    # You can't do this.
    with pytest.raises(BeartypeCallHintParamViolation):
        S.em(T.ImageObject(content_url=""))  # type: ignore


def test_link(data_regression):
    t = S.lnk("Example", "https://example.com")
    data_regression.check(unstructure(t))


def test_paragraph():
    p1 = T.Paragraph(content=[T.Text(value="Hi "), T.Text(value="There!")])
    p2 = S.p(["Hi ", "There!"])
    assert p1 == p2

    p1 = S.p("Hello ", S.em("Cruel"), " World")
    p2 = S.p(["Hello ", S.em("Cruel"), " World"])
    assert p1 == p2

    with pytest.raises(BeartypeCallHintParamViolation):
        S.p(NotAnInline("Not OK"))  # type: ignore

    with pytest.raises(BeartypeCallHintParamViolation):
        S.p("Ok", NotAnInline("Not Ok"))  # type: ignore


def test_quote(data_regression):
    t = S.qi("Methinks it is a weasel")
    data_regression.check(unstructure(t))
