# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(init=False)
class Date(Entity):
    """
    A calendar date encoded as a ISO 8601 string.
    """

    type: Literal["Date"] = field(default="Date", init=False)

    value: str
    """The date as an ISO 8601 string."""

    def __init__(self, value: str, id: Optional[str] = None):
        super().__init__(id = id)
        self.value = value
