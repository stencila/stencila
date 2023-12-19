# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class AutomaticExecution(StrEnum):
    """
    Under which circumstances the document node should be automatically executed.
    """

    Never = "Never"
    Needed = "Needed"
    Always = "Always"
