from dataclasses import dataclass
from functools import partial
from typing import Iterable, Iterator, Union

import pytest
from beartype.roar import BeartypeCallHintParamViolation

from stencila import shortcuts as S  # noqa: N812
from stencila import stencila_types as T  # noqa: N812
from stencila.convert import to_string

TOS = partial(to_string, format="md")


# TODO: We should really use hypthesis to generate these tests
def lots_of_numbers() -> Iterable[str]:
    for i in range(10):
        yield str(i) + ", "


def lots_of_numbers_and_stuff() -> Iterator[Union[str, T.Inline]]:
    a_space = T.Text(value=" ")

    yield from lots_of_numbers()

    yield T.Link(target="https://example.com", content=[T.Text(value="Example")])
    yield a_space
    yield T.Strong(content=[T.Text(value="So Strong")])
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
    assert await TOS(S.em(T.Text(value="Banana"))) == "_Banana_"

    with pytest.raises(BeartypeCallHintParamViolation):
        await TOS(S.em(T.ImageObject(content_url="")))  # type: ignore


async def test_paragraph():
    # Inline
    assert await TOS(S.p(S.em("Banana"))) == "_Banana_"

    # String
    assert await TOS(S.p("Banana")) == "Banana"

    # *args
    assert await TOS(S.p("Hello ", S.em("Cruel"), " World")) == "Hello _Cruel_ World"

    # TODO: Get the typing right on p() for these to work!
    #
    # List
    assert await TOS(S.p(["Hello ", S.em("Cruel"), " World"])) == "Hello _Cruel_ World"

    # Iterable!
    assert (
        await TOS(S.p(lots_of_numbers_and_stuff()))
        == "0, 1, 2, 3, 4, 5, 6, 7, 8, 9, [Example](https://example.com)"
        " **So Strong** 121 3.141529 true"
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


async def test_dei():
    assert (
        await TOS(S.dei("Methinks it is a weasel")) == "{--Methinks it is a weasel--}"
    )
    assert (
        await TOS(S.dei(S.ce("print(5)", lang="python")))
        == "{--`print(5)`{python exec}--}"
    )
