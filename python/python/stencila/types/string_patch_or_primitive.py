# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Primitive = ForwardRef("Primitive")
StringPatch = ForwardRef("StringPatch")


StringPatchOrPrimitive = Union[
    StringPatch,
    Primitive,
]
"""
`StringPatch` or `Primitive`
"""
