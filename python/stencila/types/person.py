# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .image_object_or_str import ImageObjectOrStr
from .organization import Organization
from .organization_or_person import OrganizationOrPerson
from .postal_address_or_str import PostalAddressOrStr
from .property_value_or_str import PropertyValueOrStr


class Person(BaseModel):
    """
    A person (alive, dead, undead, or fictional).
    """

    id: Optional[str]
    """The identifier for this item"""

    alternate_names: Optional[List[str]]
    """Alternate names (aliases) for the item."""

    description: Optional[List[Block]]
    """A description of the item."""

    identifiers: Optional[List[PropertyValueOrStr]]
    """Any kind of identifier for any kind of Thing."""

    images: Optional[List[ImageObjectOrStr]]
    """Images of the item."""

    name: Optional[str]
    """The name of the item."""

    url: Optional[str]
    """The URL of the item."""

    address: Optional[PostalAddressOrStr]
    """Postal address for the person."""

    affiliations: Optional[List[Organization]]
    """Organizations that the person is affiliated with."""

    emails: Optional[List[str]]
    """Email addresses for the person."""

    family_names: Optional[List[str]]
    """Family name. In the U.S., the last name of a person."""

    funders: Optional[List[OrganizationOrPerson]]
    """A person or organization that supports (sponsors) something through some kind of financial contribution."""

    given_names: Optional[List[str]]
    """Given name. In the U.S., the first name of a person."""

    honorific_prefix: Optional[str]
    """An honorific prefix preceding a person's name such as Dr/Mrs/Mr."""

    honorific_suffix: Optional[str]
    """An honorific suffix after a person's name such as MD/PhD/MSCSW."""

    job_title: Optional[str]
    """The job title of the person (for example, Financial Manager)."""

    member_of: Optional[List[Organization]]
    """An organization (or program membership) to which this person belongs."""

    telephone_numbers: Optional[List[str]]
    """Telephone numbers for the person."""
