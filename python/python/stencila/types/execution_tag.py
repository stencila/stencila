# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(init=False)
class ExecutionTag(Entity):
    """
    A tag on code that affects its execution.
    """

    type: Literal["ExecutionTag"] = field(default="ExecutionTag", init=False)

    name: str
    """The name of the tag"""

    value: str
    """The value of the tag"""

    is_global: bool
    """Whether the tag is global to the document"""

    def __init__(self, name: str, value: str, is_global: bool, id: Optional[str] = None):
        super().__init__(id = id)
        self.name = name
        self.value = value
        self.is_global = is_global
