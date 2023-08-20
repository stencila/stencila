# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Directory = ForwardRef("Directory")
File = ForwardRef("File")


FileOrDirectory = Union[
    File,
    Directory,
]
"""
`File` or `Directory`
"""
