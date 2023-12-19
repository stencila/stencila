# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity
from ._string_patch_or_primitive import StringPatchOrPrimitive


@dataclass(init=False)
class ModifyOperation(Entity):
    """
    An operation that is part of a suggestion to modify the property of a node.
    """

    type: Literal["ModifyOperation"] = field(default="ModifyOperation", init=False)

    target: str
    """The target property of each node to be modified."""

    value: StringPatchOrPrimitive
    """The new value, or string patch, to apply to the target property."""

    def __init__(self, target: str, value: StringPatchOrPrimitive, id: Optional[str] = None):
        super().__init__(id = id)
        self.target = target
        self.value = value
