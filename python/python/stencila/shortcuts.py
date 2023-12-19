from typing import Union, Iterable, Optional

from beartype import beartype

# Yes, do this. Why? Because having all the types around enables beartype to resolve the ForwardRefs that are
# throughout the generated code.
from .types import *


# Our convertible type handles anything that is iterable (lists, tuples, generators etc.)
try:
    # Python 3.10+
    from typing import TypeAlias

    ConvertibleToInline: TypeAlias = Union[str, Inline, Iterable[Union[Inline, str]]]
    ConvertibleToBlocks: TypeAlias = Union[str, Block, Iterable[Union[Block, str]]]
except ImportError:
    ConvertibleToInline = Union[str, Inline, Iterable[Union[Inline, str]]]
    ConvertibleToBlocks = Union[str, Block, Iterable[Union[Block, str]]]


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


@beartype
def convert_to_blocks(args: ConvertibleToBlocks):
    return [
        Paragraph(content=[Text(arg)]) if isinstance(arg, str) else arg
        for arg in flatten(args)
    ]


# TODO: Finish coping the types from short.rs, in the same order.
@beartype
def aud(url: str) -> AudioObject:
    return AudioObject(content_url=url)


# Force keywords, otherwise it is ambiguous
@beartype
def btn(*, name: str, code: str) -> Button:
    return Button(name=name, code=code)


@beartype
def ct(target: str):
    return Cite(target=target, citation_mode=CitationMode.Parenthetical)


@beartype
def ctg(targets: Iterable[str]):
    return CiteGroup(items=[ct(t) for t in targets])


@beartype
def ce(code: str, *, lang: Optional[str] = None) -> CodeExpression:
    return CodeExpression(code=code, programming_language=lang)


@beartype
def em(content: Union[str, Text]) -> Emphasis:
    # Emphasis actually takes a list, but is these are shortcuts...
    if isinstance(content, str):
        content = Text(value=content)
    return Emphasis(content=[content])


@beartype
def h(level: int, content: ConvertibleToInline):
    return Heading(level=level, content=convert_to_inlines(content))


@beartype
def lnk(content: ConvertibleToInline, target: str) -> Link:
    return Link(convert_to_inlines(content), target)


@beartype
def p(*args: ConvertibleToInline) -> Paragraph:
    return Paragraph(content=convert_to_inlines(args))


@beartype
def qi(content: ConvertibleToInline) -> QuoteInline:
    return QuoteInline(convert_to_inlines(content))


@beartype
def sec(*args: ConvertibleToBlocks) -> Section:
    return Section(content=convert_to_blocks(args))


@beartype
def fig(*args: ConvertibleToBlocks) -> Figure:
    return Figure(content=convert_to_blocks(args))
