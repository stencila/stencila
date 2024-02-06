# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

AuthorRole = ForwardRef("AuthorRole")
Organization = ForwardRef("Organization")
Person = ForwardRef("Person")
SoftwareApplication = ForwardRef("SoftwareApplication")


Author = Union[
    Person,
    Organization,
    SoftwareApplication,
    AuthorRole,
]
"""
Union type for things that can be an author of a `CreativeWork` or other type.
"""
