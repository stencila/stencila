# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class FormDeriveAction(StrEnum):
    """
    Indicates the action (create, update or delete) to derive for a `Form`.
    """

    Create = "Create"
    Update = "Update"
    Delete = "Delete"
    UpdateOrDelete = "UpdateOrDelete"
