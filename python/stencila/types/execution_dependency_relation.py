# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ExecutionDependencyRelation(StrEnum):
    """
    The relation between a node and its execution dependency.
    """

    Calls = "Calls"
    Derives = "Derives"
    Imports = "Imports"
    Includes = "Includes"
    Reads = "Reads"
    Uses = "Uses"
