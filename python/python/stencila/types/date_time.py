# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class DateTime(Entity):
    """
    A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
    """

    type: Literal["DateTime"] = field(default="DateTime", init=False)

    value: str
    """The date as an ISO 8601 string."""
