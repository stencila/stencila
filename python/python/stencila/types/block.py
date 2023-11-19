# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Admonition = ForwardRef("Admonition")
Call = ForwardRef("Call")
Claim = ForwardRef("Claim")
CodeBlock = ForwardRef("CodeBlock")
CodeChunk = ForwardRef("CodeChunk")
Figure = ForwardRef("Figure")
For = ForwardRef("For")
Form = ForwardRef("Form")
Heading = ForwardRef("Heading")
If = ForwardRef("If")
Include = ForwardRef("Include")
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
    Call,
    Claim,
    CodeBlock,
    CodeChunk,
    Figure,
    For,
    Form,
    Heading,
    If,
    Include,
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
