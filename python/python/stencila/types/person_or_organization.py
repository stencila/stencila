# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Organization = ForwardRef("Organization")
Person = ForwardRef("Person")


PersonOrOrganization = Union[
    Person,
    Organization,
]
"""
`Person` or `Organization`
"""
