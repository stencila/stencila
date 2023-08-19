# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .call import Call
from .code_chunk import CodeChunk
from .code_expression import CodeExpression
from .division import Division
from .file import File
from .for_ import For
from .if_ import If
from .span import Span
from .variable import Variable


ExecutionDependantTarget = Union[
    Call,
    CodeChunk,
    CodeExpression,
    Division,
    If,
    File,
    For,
    Span,
    Variable,
]
"""
Node types that can be execution dependants
"""
