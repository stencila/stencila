# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .grant import Grant
from .monetary_grant import MonetaryGrant


GrantOrMonetaryGrant = Union[
    Grant,
    MonetaryGrant,
]
"""
`Grant` or `MonetaryGrant`
"""
