# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .call import Call
from .claim import Claim
from .code_block import CodeBlock
from .code_chunk import CodeChunk
from .division import Division
from .figure import Figure
from .for_ import For
from .form import Form
from .heading import Heading
from .if_ import If
from .include import Include
from .list import List
from .math_block import MathBlock
from .paragraph import Paragraph
from .quote_block import QuoteBlock
from .table import Table
from .thematic_break import ThematicBreak


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
    Table,
    ThematicBreak,
]
"""
Union type for block content node types.
"""
