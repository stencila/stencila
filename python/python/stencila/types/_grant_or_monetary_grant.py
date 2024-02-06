# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Grant = ForwardRef("Grant")
MonetaryGrant = ForwardRef("MonetaryGrant")


GrantOrMonetaryGrant = Union[
    Grant,
    MonetaryGrant,
]
"""
`Grant` or `MonetaryGrant`
"""
