# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .array import Array
from .null import Null
from .object import Object
from .unsigned_integer import UnsignedInteger


Primitive = Union[
    Null,
    bool,
    int,
    UnsignedInteger,
    float,
    str,
    Array,
    Object,
]
"""
Union type for all primitives values
"""
