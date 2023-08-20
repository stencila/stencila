# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

Organization = ForwardRef("Organization")
from .organization_or_person import OrganizationOrPerson
from .postal_address_or_str import PostalAddressOrStr
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class Person(Thing):
    """
    A person (alive, dead, undead, or fictional).
    """

    type: Literal["Person"] = field(default="Person", init=False)

    address: Optional[PostalAddressOrStr] = None
    """Postal address for the person."""

    affiliations: Optional[List[Organization]] = None
    """Organizations that the person is affiliated with."""

    emails: Optional[List[str]] = None
    """Email addresses for the person."""

    family_names: Optional[List[str]] = None
    """Family name. In the U.S., the last name of a person."""

    funders: Optional[List[OrganizationOrPerson]] = None
    """A person or organization that supports (sponsors) something through some kind of financial contribution."""

    given_names: Optional[List[str]] = None
    """Given name. In the U.S., the first name of a person."""

    honorific_prefix: Optional[str] = None
    """An honorific prefix preceding a person's name such as Dr/Mrs/Mr."""

    honorific_suffix: Optional[str] = None
    """An honorific suffix after a person's name such as MD/PhD/MSCSW."""

    job_title: Optional[str] = None
    """The job title of the person (for example, Financial Manager)."""

    member_of: Optional[List[Organization]] = None
    """An organization (or program membership) to which this person belongs."""

    telephone_numbers: Optional[List[str]] = None
    """Telephone numbers for the person."""
