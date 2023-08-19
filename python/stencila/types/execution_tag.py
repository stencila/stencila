# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ExecutionTag(BaseModel):
    """
    A tag on code that affects its execution
    """

    name: str
    """The name of the tag"""

    value: str
    """The value of the tag"""

    is_global: bool
    """Whether the tag is global to the document"""
