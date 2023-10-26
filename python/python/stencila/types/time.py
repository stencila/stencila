# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(init=False)
class Time(Entity):
    """
    A point in time recurring on multiple days.
    """

    type: Literal["Time"] = field(default="Time", init=False)

    value: str
    """The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`."""

    def __init__(self, value: str, id: Optional[str] = None):
        super().__init__(id = id)
        self.value = value
