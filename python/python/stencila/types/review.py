# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .creative_work import CreativeWork
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class Review(CreativeWork):
    """
    A review of an item, e.g of an `Article` or `SoftwareApplication`.
    """

    type: Literal["Review"] = field(default="Review", init=False)

    item_reviewed: Optional[Thing] = None
    """The item that is being reviewed."""

    review_aspect: Optional[str] = None
    """The part or facet of the item that is being reviewed."""
