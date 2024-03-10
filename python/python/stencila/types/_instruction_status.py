# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class InstructionStatus(StrEnum):
    """
    The status of an instruction.
    """

    Proposed = "Proposed"
    Accepted = "Accepted"
    RevisionsRequested = "RevisionsRequested"
    Rejected = "Rejected"
