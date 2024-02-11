# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ArrayHint = ForwardRef("ArrayHint")
Function = ForwardRef("Function")
ObjectHint = ForwardRef("ObjectHint")
StringHint = ForwardRef("StringHint")
Unknown = ForwardRef("Unknown")


Hint = Union[
    ArrayHint,
    Function,
    ObjectHint,
    StringHint,
    Unknown,
    bool,
    int,
    float,
]
"""
Union type for hints of the value and/or structure of data.
"""
