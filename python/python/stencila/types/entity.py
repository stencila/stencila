# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


@dataclass(kw_only=True, frozen=True)
class Entity(DataClassJsonMixin):
    """
    Abstract base type for compound (ie. non-atomic) nodes.
    """

    type: Literal["Entity"] = field(default="Entity", init=False)

    id: Optional[str] = None
    """The identifier for this item."""
