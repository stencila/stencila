from dataclasses import dataclass
from functools import partial
from typing import Iterable, Union, Iterator

import pytest
from beartype.roar import BeartypeCallHintParamViolation

from stencila import shortcuts as S, types as T
from stencila.convert import to_string

TOS = partial(to_string, format="md")


# TODO: Make these fixtures, maybe.
def lots_of_numbers() -> Iterable[str]:
    for i in range(10):
        yield str(i) + ", "


def lots_of_numbers_and_stuff() -> Iterator[Union[str, T.Inline]]:
    a_space = T.Text(" ")

    yield from lots_of_numbers()

    yield T.Link(target="https://example.com", content=[T.Text("Example")])
    yield a_space
    yield T.Strong(content=[T.Text("So Strong")])
    yield a_space
    yield 11 * 11
    yield a_space
    yield 3.141529
    yield a_space
    yield True


@dataclass
class NotAnInline:
    content: str


# Start testing -----------------
async def test_emphasis():
    assert await TOS(S.em("Banana")) == "_Banana_"
    assert await TOS(S.em(T.Text("Banana"))) == "_Banana_"

    with pytest.raises(BeartypeCallHintParamViolation):
        await TOS(S.em(T.ImageObject(content_url="")))  # type: ignore


async def test_paragraph():
    # Inline
    assert await TOS(S.p(S.em("Banana"))) == "_Banana_"

    # String
    assert await TOS(S.p("Banana")) == "Banana"

    # *args
    assert await TOS(S.p("Hello ", S.em("Cruel"), " World")) == "Hello _Cruel_ World"

    # List
    assert await TOS(S.p(["Hello ", S.em("Cruel"), " World"])) == "Hello _Cruel_ World"

    # Iterable!
    assert (
        await TOS(S.p(lots_of_numbers_and_stuff()))
        == "0, 1, 2, 3, 4, 5, 6, 7, 8, 9, [Example](https://example.com) **So Strong** 121 3.141529 true"
    )

    with pytest.raises(BeartypeCallHintParamViolation):
        await TOS(S.p(NotAnInline("Not OK")))  # type: ignore

    with pytest.raises(BeartypeCallHintParamViolation):
        await TOS(S.p("Ok", NotAnInline("Not Ok")))  # type: ignore


async def test_link():
    assert (
        await TOS(S.lnk("Example", "https://example.com"))
        == "[Example](https://example.com)"
    )


async def test_quote():
    assert (
        await TOS(S.qi("Methinks it is a weasel")) == "<q>Methinks it is a weasel</q>"
    )
