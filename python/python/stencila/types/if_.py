# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .executable import Executable
from .if_clause import IfClause


@dataclass(kw_only=True, frozen=True)
class If(Executable):
    """
    Show and execute alternative content conditional upon an executed expression.
    """

    type: Literal["If"] = field(default="If", init=False)

    clauses: List[IfClause]
    """The clauses making up the `If` node"""
