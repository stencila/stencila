from typing import Iterable, Optional, Union

from beartype import beartype

# Yes, do this. Why? Because having all the types around enables beartype to
# resolve the ForwardRefs that are throughout the generated code.
from .types import *

# Our convertible type handles anything that is iterable (lists, tuples,
# generators etc.)
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
def ci(code: str):
    return CodeInline(code)


@beartype
def dei(content: ConvertibleToInline) -> DeleteInline:
    return DeleteInline(convert_to_inlines(content))


@beartype
def em(content: Union[str, Text]) -> Emphasis:
    # Emphasis actually takes a list, but if these are shortcuts...
    if isinstance(content, str):
        content = Text(value=content)
    return Emphasis(content=[content])


@beartype
def img(url: str) -> ImageObject:
    return ImageObject(content_url=url)


@beartype
def isi(content: ConvertibleToInline) -> InsertInline:
    return InsertInline(convert_to_inlines(content))


@beartype
def lnk(content: ConvertibleToInline, target: str) -> Link:
    return Link(convert_to_inlines(content), target)


@beartype
def mi(code: str, *, lang: str | None = None) -> MathInline:
    return MathInline(code, math_language=lang)


@beartype
def nte(note_type: NoteType, content: ConvertibleToBlocks) -> Note:
    return Note(note_type, convert_to_blocks(content))


@beartype
def par(name: str) -> Parameter:
    return Parameter(name)


@beartype
def rei(
    content: ConvertibleToInline, replacement: ConvertibleToInline
) -> ReplaceInline:
    return ReplaceInline(convert_to_inlines(content), convert_to_inlines(replacement))


@beartype
def qi(content: ConvertibleToInline) -> QuoteInline:
    return QuoteInline(convert_to_inlines(content))


@beartype
def sti(code: str, content: ConvertibleToInline) -> StyledInline:
    return StyledInline(code, convert_to_inlines(content))


@beartype
def stk(content: ConvertibleToInline) -> Strong:
    return Strong(convert_to_inlines(content))


@beartype
def sub(content: ConvertibleToInline) -> Subscript:
    return Subscript(convert_to_inlines(content))


@beartype
def sup(content: ConvertibleToInline) -> Superscript:
    return Superscript(convert_to_inlines(content))


@beartype
def t(content: ConvertibleToInline) -> Text:
    return Text(convert_to_inlines(content))


@beartype
def ul(content: ConvertibleToInline) -> Underline:
    return Underline(convert_to_inlines(content))


@beartype
def stg(content: ConvertibleToInline) -> Strikeout:
    return Strikeout(convert_to_inlines(content))


@beartype
def stg(url: str) -> VideoObject:
    return VideoObject(url)


@beartype
def adm(
    admonition_type: AdmonitionType,
    content: ConvertibleToBlocks,
    *,
    title: str | None = None,
) -> Admonition:
    return Admonition(admonition_type, convert_to_blocks(content), title=title)


@beartype
def clb(source: str, args: list[CallArgument]) -> CallBlock:
    return CallBlock(source, arguments=args)


@beartype
def arg(name: str, code: str) -> CallArgument:
    return CallArgument(name, code)


@beartype
def clm(claim_type: ClaimType, content: ConvertibleToBlocks) -> Claim:
    return Claim(claim_type, convert_to_blocks(content))


@beartype
def cb(code: str, *, lang: str | None = None) -> CodeBlock:
    return CodeBlock(code, programming_language=lang)


@beartype
def cc(code: str, *, lang: str | None = None) -> CodeChunk:
    return CodeChunk(code, programming_language=lang)


@beartype
def deb(content: ConvertibleToBlocks) -> DeleteBlock:
    return DeleteBlock(convert_to_blocks(content))


@beartype
def fig(content: ConvertibleToBlocks) -> Figure:
    return Figure(convert_to_blocks(content))


@beartype
def frb(code: str, symbol: str, content: ConvertibleToBlocks) -> ForBlock:
    return ForBlock(code=code, symbol=symbol, content=convert_to_blocks(content))


@beartype
def h(level: int, content: ConvertibleToInline):
    return Heading(level=level, content=convert_to_inlines(content))


@beartype
def h1(content: ConvertibleToInline) -> Heading:
    return h(1, content)


@beartype
def h2(content: ConvertibleToInline) -> Heading:
    return h(2, content)


@beartype
def h3(content: ConvertibleToInline) -> Heading:
    return h(3, content)


@beartype
def h4(content: ConvertibleToInline) -> Heading:
    return h(4, content)


@beartype
def h5(content: ConvertibleToInline) -> Heading:
    return h(5, content)


@beartype
def h6(content: ConvertibleToInline) -> Heading:
    return h(6, content)


@beartype
def ifb(clauses: list[IfBlockClause]) -> IfBlock:
    return IfBlock(clauses)


@beartype
def ibc(
    code: str, content: ConvertibleToBlocks, *, lang: str | None = None
) -> IfBlockClause:
    return IfBlockClause(
        code=code, programming_language=lang, content=convert_to_blocks(content)
    )


@beartype
def inb(source: str) -> IncludeBlock:
    return IncludeBlock(source=source)


@beartype
def isb(content: ConvertibleToBlocks) -> InsertBlock:
    return InsertBlock(content=convert_to_blocks(content))


@beartype
def ol(items: ListItem) -> List_:
    return List_(items, order=ListOrder.Ascending)


@beartype
def ul(items: ListItem) -> List_:
    return List_(items, order=ListOrder.Unordered)


@beartype
def li(content: ConvertibleToInline) -> ListItem:
    return ListItem(convert_to_inlines(content))


@beartype
def mb(code: str, *, lang: str | None = None) -> MathBlock:
    return MathBlock(convert_to_blocks(code), math_language=lang)


@beartype
def p(*args: ConvertibleToInline) -> Paragraph:
    return Paragraph(content=convert_to_inlines(args))


@beartype
def qb(content: ConvertibleToBlocks) -> QuoteBlock:
    return QuoteBlock(convert_to_blocks(content))


@beartype
def reb(content: ConvertibleToBlocks, replacement: ConvertibleToBlocks) -> ReplaceBlock:
    return ReplaceBlock(convert_to_blocks(content), convert_to_blocks(replacement))


@beartype
def sec(content: ConvertibleToBlocks) -> Section:
    return Section(convert_to_blocks(content))


@beartype
def stb(content: ConvertibleToBlocks) -> StyledBlock:
    return StyledBlock(convert_to_blocks(content))


@beartype
def tbl(rows: list[TableRow]) -> Table:
    return Table(rows)


@beartype
def tr(cells: list[TableCell]) -> TableRow:
    return TableRow(cells)


@beartype
def th(content: ConvertibleToInline) -> TableCell:
    return TableCell(
        content=convert_to_inlines(content), cell_type=TableCellType.HeaderCell
    )


@beartype
def td(content: ConvertibleToInline) -> TableCell:
    return TableCell(
        content=convert_to_inlines(content), cell_type=TableCellType.DataCell
    )


@beartype
def tb() -> ThematicBreak:
    return ThematicBreak()


@beartype
def art(content: ConvertibleToBlocks) -> Article:
    return Article(convert_to_blocks(content))
