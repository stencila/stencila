# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .string_operation import StringOperation


@dataclass(init=False)
class StringPatch(Entity):
    """
    An set of operations to modify a string.
    """

    type: Literal["StringPatch"] = field(default="StringPatch", init=False)

    version: Optional[str] = None
    """The version of the string to which the patch should be applied."""

    operations: List[StringOperation]
    """The operations to be applied to the string."""

    def __init__(self, operations: List[StringOperation], id: Optional[str] = None, version: Optional[str] = None):
        super().__init__(id = id)
        self.version = version
        self.operations = operations
