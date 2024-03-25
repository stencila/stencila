from collections.abc import Iterable
from typing import TypeAlias

from beartype import beartype

from stencila_types import types as T  # noqa: N812

# Our convertible type handles anything that is iterable (lists, tuples,
# generators etc.)

ConvertibleToInline: TypeAlias = str | T.Inline | Iterable[T.Inline | str]
ConvertibleToBlocks: TypeAlias = str | T.Block | Iterable[T.Block | str]


def flatten(args):
    """This makes our job much easier by normalising everything to an iterable."""
    if isinstance(args, str) or not isinstance(args, Iterable):
        yield args
    else:
        for elem in args:
            yield from flatten(elem)


@beartype
def convert_to_inlines(args: ConvertibleToInline):
    return [T.Text(value=arg) if isinstance(arg, str) else arg for arg in flatten(args)]


@beartype
def convert_to_blocks(args: ConvertibleToBlocks):
    return [
        T.Paragraph(content=[T.Text(value=arg)]) if isinstance(arg, str) else arg
        for arg in flatten(args)
    ]


@beartype
def aud(url: str) -> T.AudioObject:
    return T.AudioObject(content_url=url)


# Force keywords, otherwise it is ambiguous
@beartype
def btn(*, name: str, code: str) -> T.Button:
    return T.Button(name=name, code=code)


@beartype
def ct(target: str):
    return T.Cite(target=target, citation_mode=T.CitationMode.Parenthetical)


@beartype
def ctg(targets: Iterable[str]):
    return T.CiteGroup(items=[ct(t) for t in targets])


@beartype
def ce(code: str, *, lang: str | None = None) -> T.CodeExpression:
    return T.CodeExpression(code=code, programming_language=lang)


@beartype
def ci(code: str):
    return T.CodeInline(code=code)


@beartype
def dei(content: ConvertibleToInline) -> T.DeleteInline:
    return T.DeleteInline(content=convert_to_inlines(content))


@beartype
def em(content: str | T.Text) -> T.Emphasis:
    # Emphasis actually takes a list, but if these are shortcuts...
    if isinstance(content, str):
        content = T.Text(value=content)
    return T.Emphasis(content=[content])


@beartype
def img(url: str) -> T.ImageObject:
    return T.ImageObject(content_url=url)


@beartype
def isi(content: ConvertibleToInline) -> T.InsertInline:
    return T.InsertInline(content=convert_to_inlines(content))


@beartype
def lnk(content: ConvertibleToInline, target: str) -> T.Link:
    return T.Link(content=convert_to_inlines(content), target=target)


@beartype
def mi(code: str, *, lang: str | None = None) -> T.MathInline:
    return T.MathInline(code=code, math_language=lang)


@beartype
def nte(note_type: T.NoteType, content: ConvertibleToBlocks) -> T.Note:
    return T.Note(note_type=note_type, content=convert_to_blocks(content))


@beartype
def par(name: str) -> T.Parameter:
    return T.Parameter(name=name)


@beartype
def rei(
    content: ConvertibleToInline, replacement: ConvertibleToInline
) -> T.ReplaceInline:
    return T.ReplaceInline(
        content=convert_to_inlines(content),
        replacement=convert_to_inlines(replacement),
    )


@beartype
def qi(content: ConvertibleToInline) -> T.QuoteInline:
    return T.QuoteInline(content=convert_to_inlines(content))


@beartype
def sti(code: str, content: ConvertibleToInline) -> T.StyledInline:
    return T.StyledInline(code=code, content=convert_to_inlines(content))


@beartype
def stk(content: ConvertibleToInline) -> T.Strikeout:
    return T.Strikeout(content=convert_to_inlines(content))


@beartype
def stg(content: ConvertibleToInline) -> T.Strong:
    return T.Strong(content=convert_to_inlines(content))


@beartype
def sub(content: ConvertibleToInline) -> T.Subscript:
    return T.Subscript(content=convert_to_inlines(content))


@beartype
def sup(content: ConvertibleToInline) -> T.Superscript:
    return T.Superscript(content=convert_to_inlines(content))


@beartype
def t(content: str) -> T.Text:
    return T.Text(value=str(content))


@beartype
def u(content: ConvertibleToInline) -> T.Underline:
    return T.Underline(content=convert_to_inlines(content))


@beartype
def adm(
    admonition_type: T.AdmonitionType,
    content: ConvertibleToBlocks,
    *,
    title: str | None = None,
) -> T.Admonition:
    return T.Admonition(
        admonition_type=admonition_type,
        content=convert_to_blocks(content),
        title=convert_to_inlines(title),
    )


@beartype
def clb(source: str, args: list[T.CallArgument]) -> T.CallBlock:
    return T.CallBlock(source=source, arguments=args)


@beartype
def arg(name: str, code: str) -> T.CallArgument:
    return T.CallArgument(name=name, code=code)


@beartype
def clm(claim_type: T.ClaimType, content: ConvertibleToBlocks) -> T.Claim:
    return T.Claim(
        claim_type=claim_type,
        content=convert_to_blocks(content),
    )


@beartype
def cb(code: str, *, lang: str | None = None) -> T.CodeBlock:
    return T.CodeBlock(code=code, programming_language=lang)


@beartype
def cc(code: str, *, lang: str | None = None) -> T.CodeChunk:
    return T.CodeChunk(code=code, programming_language=lang)


@beartype
def deb(content: ConvertibleToBlocks) -> T.DeleteBlock:
    return T.DeleteBlock(content=convert_to_blocks(content))


@beartype
def fig(content: ConvertibleToBlocks) -> T.Figure:
    return T.Figure(content=convert_to_blocks(content))


@beartype
def frb(code: str, variable: str, content: ConvertibleToBlocks) -> T.ForBlock:
    return T.ForBlock(code=code, variable=variable, content=convert_to_blocks(content))


@beartype
def h(level: int, content: ConvertibleToInline):
    return T.Heading(level=level, content=convert_to_inlines(content))


@beartype
def h1(content: ConvertibleToInline) -> T.Heading:
    return h(1, content)


@beartype
def h2(content: ConvertibleToInline) -> T.Heading:
    return h(2, content)


@beartype
def h3(content: ConvertibleToInline) -> T.Heading:
    return h(3, content)


@beartype
def h4(content: ConvertibleToInline) -> T.Heading:
    return h(4, content)


@beartype
def h5(content: ConvertibleToInline) -> T.Heading:
    return h(5, content)


@beartype
def h6(content: ConvertibleToInline) -> T.Heading:
    return h(6, content)


@beartype
def ifb(clauses: list[T.IfBlockClause]) -> T.IfBlock:
    return T.IfBlock(clauses=clauses)


@beartype
def ibc(
    code: str, content: ConvertibleToBlocks, *, lang: str | None = None
) -> T.IfBlockClause:
    return T.IfBlockClause(
        code=code, programming_language=lang, content=convert_to_blocks(content)
    )


@beartype
def inb(source: str) -> T.IncludeBlock:
    return T.IncludeBlock(source=source)


@beartype
def isb(content: ConvertibleToBlocks) -> T.InsertBlock:
    return T.InsertBlock(content=convert_to_blocks(content))


@beartype
def ol(items: list[T.ListItem]) -> T.List:
    return T.List(items=items, order=T.ListOrder.Ascending)


@beartype
def ul(items: list[T.ListItem]) -> T.List:
    return T.List(items=items, order=T.ListOrder.Unordered)


@beartype
def li(content: ConvertibleToBlocks) -> T.ListItem:
    return T.ListItem(content=convert_to_blocks(content))


@beartype
def mb(code: str, *, lang: str | None = None) -> T.MathBlock:
    return T.MathBlock(code=code, math_language=lang)


@beartype
def p(*args: ConvertibleToInline) -> T.Paragraph:
    return T.Paragraph(content=convert_to_inlines(args))


@beartype
def qb(content: ConvertibleToBlocks) -> T.QuoteBlock:
    return T.QuoteBlock(content=convert_to_blocks(content))


@beartype
def reb(
    content: ConvertibleToBlocks, replacement: ConvertibleToBlocks
) -> T.ReplaceBlock:
    return T.ReplaceBlock(
        content=convert_to_blocks(content),
        replacement=convert_to_blocks(replacement),
    )


@beartype
def sec(content: ConvertibleToBlocks) -> T.Section:
    return T.Section(content=convert_to_blocks(content))


@beartype
def stb(style: str, content: ConvertibleToBlocks) -> T.StyledBlock:
    return T.StyledBlock(code=style, content=convert_to_blocks(content))


@beartype
def tbl(rows: list[T.TableRow]) -> T.Table:
    return T.Table(rows=rows)


@beartype
def tr(cells: list[T.TableCell]) -> T.TableRow:
    return T.TableRow(cells=cells)


@beartype
def th(content: ConvertibleToBlocks) -> T.TableCell:
    return T.TableCell(
        content=convert_to_blocks(content),
        cell_type=T.TableCellType.HeaderCell,
    )


@beartype
def td(content: ConvertibleToBlocks) -> T.TableCell:
    return T.TableCell(
        content=convert_to_blocks(content),
        cell_type=T.TableCellType.DataCell,
    )


@beartype
def tb() -> T.ThematicBreak:
    return T.ThematicBreak()


@beartype
def art(content: ConvertibleToBlocks) -> T.Article:
    return T.Article(content=convert_to_blocks(content))
