# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity
from ._string_operation import StringOperation


@dataclass(init=False)
class StringPatch(Entity):
    """
    An set of operations to modify a string.
    """

    type: Literal["StringPatch"] = field(default="StringPatch", init=False)

    operations: List[StringOperation]
    """The operations to be applied to the string."""

    def __init__(self, operations: List[StringOperation], id: Optional[str] = None):
        super().__init__(id = id)
        self.operations = operations
