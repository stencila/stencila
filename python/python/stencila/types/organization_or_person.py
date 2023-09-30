# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Organization = ForwardRef("Organization")
Person = ForwardRef("Person")


OrganizationOrPerson = Union[
    Organization,
    Person,
]
"""
`Organization` or `Person`
"""
