# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Call = ForwardRef("Call")
Claim = ForwardRef("Claim")
CodeBlock = ForwardRef("CodeBlock")
CodeChunk = ForwardRef("CodeChunk")
Division = ForwardRef("Division")
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
Table = ForwardRef("Table")
ThematicBreak = ForwardRef("ThematicBreak")


Block = Union[
    Call,
    Claim,
    CodeBlock,
    CodeChunk,
    Division,
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
    Table,
    ThematicBreak,
]
"""
Union type in block content node types.
"""
