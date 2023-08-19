# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .organization import Organization
from .person import Person


PersonOrOrganization = Union[
    Person,
    Organization,
]
"""
`Person` or `Organization`
"""
