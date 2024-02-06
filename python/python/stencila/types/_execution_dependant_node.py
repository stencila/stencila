# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Button = ForwardRef("Button")
CallBlock = ForwardRef("CallBlock")
CodeChunk = ForwardRef("CodeChunk")
CodeExpression = ForwardRef("CodeExpression")
File = ForwardRef("File")
Function = ForwardRef("Function")
Parameter = ForwardRef("Parameter")
StyledBlock = ForwardRef("StyledBlock")
StyledInline = ForwardRef("StyledInline")
Variable = ForwardRef("Variable")


ExecutionDependantNode = Union[
    Button,
    CallBlock,
    CodeChunk,
    CodeExpression,
    File,
    Function,
    Parameter,
    StyledBlock,
    StyledInline,
    Variable,
]
"""
Node types that can be execution dependencies.
"""
