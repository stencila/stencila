# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .modify_operation import ModifyOperation


@dataclass(init=False)
class Modify(DataClassJsonMixin):
    """
    A suggestion to modify one or more nodes.
    """

    type: Literal["Modify"] = field(default="Modify", init=False)

    operations: List[ModifyOperation]
    """The operations to be applied to the nodes."""

    def __init__(self, operations: List[ModifyOperation]):
        super().__init__()
        self.operations = operations
