# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Button = ForwardRef("Button")
Call = ForwardRef("Call")
CodeChunk = ForwardRef("CodeChunk")
CodeExpression = ForwardRef("CodeExpression")
Division = ForwardRef("Division")
File = ForwardRef("File")
Function = ForwardRef("Function")
Parameter = ForwardRef("Parameter")
StyledInline = ForwardRef("StyledInline")
Variable = ForwardRef("Variable")


ExecutionDependantNode = Union[
    Button,
    Call,
    CodeChunk,
    CodeExpression,
    Division,
    File,
    Function,
    Parameter,
    StyledInline,
    Variable,
]
"""
Node types that can be execution dependencies.
"""
