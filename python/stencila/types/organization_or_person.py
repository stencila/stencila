# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .organization import Organization
from .person import Person


OrganizationOrPerson = Union[
    Organization,
    Person,
]
"""
`Organization` or `Person`
"""
