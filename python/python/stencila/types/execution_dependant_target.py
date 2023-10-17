# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Call = ForwardRef("Call")
CodeChunk = ForwardRef("CodeChunk")
CodeExpression = ForwardRef("CodeExpression")
Division = ForwardRef("Division")
File = ForwardRef("File")
For = ForwardRef("For")
If = ForwardRef("If")
Span = ForwardRef("Span")
Variable = ForwardRef("Variable")


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
Node types that can be execution dependants.
"""
