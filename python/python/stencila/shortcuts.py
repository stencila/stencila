from typing import Union, Iterable

from beartype import beartype

# Yes, do this. Why? Because having all the types around enables beartype to resolve the ForwardRefs that are
# throughout the generated code.
from .types import *


# Our convertible type handles anything that is iterable (lists, tuples, generators etc.)
try:
    # Python 3.10+
    from typing import TypeAlias

    ConvertibleToInline: TypeAlias = Union[str, Inline, Iterable[Union[Inline, str]]]
except ImportError:
    ConvertibleToInline = Union[str, Inline, Iterable[Union[Inline, str]]]


def flatten(args):
    """This makes our job much easier by normalising everything to an iterable."""
    if isinstance(args, str) or not isinstance(args, Iterable):
        yield args
    else:
        for elem in args:
            yield from flatten(elem)


@beartype
def convert_to_inlines(args: ConvertibleToInline):
    return [Text(value=arg) if isinstance(arg, str) else arg for arg in flatten(args)]


# TODO: Copy the types from short.rs, in the same order.
# Here is a start.


@beartype
def em(content: Union[str, Text]) -> Emphasis:
    # Emphasis actually takes a list, but is these are shortcuts...
    if isinstance(content, str):
        content = Text(value=content)
    return Emphasis(content=[content])


@beartype
def lnk(content: ConvertibleToInline, target: str) -> Link:
    return Link(convert_to_inlines(content), target)


@beartype
def p(*args: ConvertibleToInline) -> Paragraph:
    return Paragraph(content=convert_to_inlines(args))


@beartype
def qi(content: ConvertibleToInline) -> QuoteInline:
    return QuoteInline(convert_to_inlines(content))
