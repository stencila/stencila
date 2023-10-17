# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Button = ForwardRef("Button")
CodeChunk = ForwardRef("CodeChunk")
File = ForwardRef("File")
Parameter = ForwardRef("Parameter")
SoftwareSourceCode = ForwardRef("SoftwareSourceCode")
Variable = ForwardRef("Variable")


ExecutionDependencyNode = Union[
    Button,
    CodeChunk,
    File,
    Parameter,
    SoftwareSourceCode,
    Variable,
]
"""
Node types that can be execution dependencies.
"""
