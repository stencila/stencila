# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class AuthorRoleName(StrEnum):
    """
    A `roleName` for an `AuthorRole`.
    """

    Writer = "Writer"
    Instructor = "Instructor"
    Prompter = "Prompter"
    Generator = "Generator"
