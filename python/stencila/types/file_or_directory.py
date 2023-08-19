# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .directory import Directory
from .file import File


FileOrDirectory = Union[
    File,
    Directory,
]
"""
`File` or `Directory`
"""
