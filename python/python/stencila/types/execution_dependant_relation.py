# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ExecutionDependantRelation(StrEnum):
    """
    The relation between a node and its execution dependant.
    """

    Assigns = "Assigns"
    Alters = "Alters"
    Declares = "Declares"
    Writes = "Writes"
