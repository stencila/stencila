# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class Date(Entity):
    """
    A calendar date encoded as a ISO 8601 string.
    """

    type: Literal["Date"] = field(default="Date", init=False)

    value: str
    """The date as an ISO 8601 string."""
