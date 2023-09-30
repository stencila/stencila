# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .person_or_organization import PersonOrOrganization
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class Grant(Thing):
    """
    A grant, typically financial or otherwise quantifiable, of resources.
    """

    type: Literal["Grant"] = field(default="Grant", init=False)

    funded_items: Optional[List[Thing]] = None
    """Indicates an item funded or sponsored through a Grant."""

    sponsors: Optional[List[PersonOrOrganization]] = None
    """A person or organization that supports a thing through a pledge, promise, or financial contribution."""
