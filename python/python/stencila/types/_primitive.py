# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Array = ForwardRef("Array")
Object = ForwardRef("Object")
UnsignedInteger = ForwardRef("UnsignedInteger")


Primitive = Union[
    None,
    bool,
    int,
    UnsignedInteger,
    float,
    str,
    Array,
    Object,
]
"""
Union type for all primitives values.
"""
