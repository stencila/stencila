# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

PostalAddress = ForwardRef("PostalAddress")


PostalAddressOrStr = Union[
    PostalAddress,
    str,
]
"""
`PostalAddress` or `str`
"""
