# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Admonition = ForwardRef("Admonition")
CallBlock = ForwardRef("CallBlock")
Claim = ForwardRef("Claim")
CodeBlock = ForwardRef("CodeBlock")
CodeChunk = ForwardRef("CodeChunk")
Figure = ForwardRef("Figure")
ForBlock = ForwardRef("ForBlock")
Form = ForwardRef("Form")
Heading = ForwardRef("Heading")
IfBlock = ForwardRef("IfBlock")
IncludeBlock = ForwardRef("IncludeBlock")
List = ForwardRef("List")
MathBlock = ForwardRef("MathBlock")
Paragraph = ForwardRef("Paragraph")
QuoteBlock = ForwardRef("QuoteBlock")
Section = ForwardRef("Section")
StyledBlock = ForwardRef("StyledBlock")
Table = ForwardRef("Table")
ThematicBreak = ForwardRef("ThematicBreak")


Block = Union[
    Admonition,
    CallBlock,
    Claim,
    CodeBlock,
    CodeChunk,
    Figure,
    ForBlock,
    Form,
    Heading,
    IfBlock,
    IncludeBlock,
    List,
    MathBlock,
    Paragraph,
    QuoteBlock,
    Section,
    StyledBlock,
    Table,
    ThematicBreak,
]
"""
Union type in block content node types.
"""
