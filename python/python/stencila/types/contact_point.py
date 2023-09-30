# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class ContactPoint(Thing):
    """
    A contact point, usually within an organization.
    """

    type: Literal["ContactPoint"] = field(default="ContactPoint", init=False)

    emails: Optional[List[str]] = None
    """Email address for correspondence."""

    telephone_numbers: Optional[List[str]] = None
    """Telephone numbers for the contact point."""

    available_languages: Optional[List[str]] = None
    """Languages (human not programming) in which it is possible to communicate with the organization/department etc."""
