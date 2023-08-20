# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .grant import Grant
from .person_or_organization import PersonOrOrganization


@dataclass(kw_only=True, frozen=True)
class MonetaryGrant(Grant):
    """
    A monetary grant.
    """

    type: Literal["MonetaryGrant"] = field(default="MonetaryGrant", init=False)

    amounts: Optional[float] = None
    """The amount of money."""

    funders: Optional[List[PersonOrOrganization]] = None
    """A person or organization that supports (sponsors) something through some kind of financial contribution."""
