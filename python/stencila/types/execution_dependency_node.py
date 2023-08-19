# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .button import Button
from .code_chunk import CodeChunk
from .file import File
from .parameter import Parameter
from .software_source_code import SoftwareSourceCode
from .variable import Variable


ExecutionDependencyNode = Union[
    Button,
    CodeChunk,
    File,
    Parameter,
    SoftwareSourceCode,
    Variable,
]
"""
Node types that can be execution dependencies
"""
