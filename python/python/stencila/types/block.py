# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Admonition = ForwardRef("Admonition")
CallBlock = ForwardRef("CallBlock")
Claim = ForwardRef("Claim")
CodeBlock = ForwardRef("CodeBlock")
CodeChunk = ForwardRef("CodeChunk")
DeleteBlock = ForwardRef("DeleteBlock")
Figure = ForwardRef("Figure")
ForBlock = ForwardRef("ForBlock")
Form = ForwardRef("Form")
Heading = ForwardRef("Heading")
IfBlock = ForwardRef("IfBlock")
IncludeBlock = ForwardRef("IncludeBlock")
InsertBlock = ForwardRef("InsertBlock")
InstructionBlock = ForwardRef("InstructionBlock")
List = ForwardRef("List")
MathBlock = ForwardRef("MathBlock")
ModifyBlock = ForwardRef("ModifyBlock")
Paragraph = ForwardRef("Paragraph")
QuoteBlock = ForwardRef("QuoteBlock")
ReplaceBlock = ForwardRef("ReplaceBlock")
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
    DeleteBlock,
    Figure,
    ForBlock,
    Form,
    Heading,
    IfBlock,
    IncludeBlock,
    InsertBlock,
    InstructionBlock,
    List,
    MathBlock,
    ModifyBlock,
    Paragraph,
    QuoteBlock,
    ReplaceBlock,
    Section,
    StyledBlock,
    Table,
    ThematicBreak,
]
"""
Union type in block content node types.
"""
