# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(kw_only=True, frozen=True)
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
