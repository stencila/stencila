# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .button import Button
from .call import Call
from .code_chunk import CodeChunk
from .code_expression import CodeExpression
from .division import Division
from .file import File
from .parameter import Parameter
from .span import Span
from .variable import Variable


ExecutionDependantNode = Union[
    Button,
    Call,
    CodeChunk,
    CodeExpression,
    Division,
    File,
    Parameter,
    Span,
    Variable,
]
"""
Node types that can be execution dependencies
"""
