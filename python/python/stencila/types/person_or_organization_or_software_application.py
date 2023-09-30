# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Organization = ForwardRef("Organization")
Person = ForwardRef("Person")
SoftwareApplication = ForwardRef("SoftwareApplication")


PersonOrOrganizationOrSoftwareApplication = Union[
    Person,
    Organization,
    SoftwareApplication,
]
"""
`Person` or `Organization` or `SoftwareApplication`
"""
