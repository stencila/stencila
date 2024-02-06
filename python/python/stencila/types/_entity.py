# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


@dataclass(init=False)
class Entity(DataClassJsonMixin):
    """
    Abstract base type for compound (ie. non-atomic) nodes.
    """

    type: Literal["Entity"] = field(default="Entity", init=False)

    id: Optional[str] = None
    """The identifier for this item."""

    def __init__(self, id: Optional[str] = None):
        super().__init__()
        self.id = id
