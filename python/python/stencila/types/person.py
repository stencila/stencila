# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ImageObject = ForwardRef("ImageObject")
Organization = ForwardRef("Organization")
from .person_or_organization import PersonOrOrganization
from .postal_address_or_str import PostalAddressOrStr
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
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

    funders: Optional[List[PersonOrOrganization]] = None
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

    def __init__(self, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, address: Optional[PostalAddressOrStr] = None, affiliations: Optional[List[Organization]] = None, emails: Optional[List[str]] = None, family_names: Optional[List[str]] = None, funders: Optional[List[PersonOrOrganization]] = None, given_names: Optional[List[str]] = None, honorific_prefix: Optional[str] = None, honorific_suffix: Optional[str] = None, job_title: Optional[str] = None, member_of: Optional[List[Organization]] = None, telephone_numbers: Optional[List[str]] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.address = address
        self.affiliations = affiliations
        self.emails = emails
        self.family_names = family_names
        self.funders = funders
        self.given_names = given_names
        self.honorific_prefix = honorific_prefix
        self.honorific_suffix = honorific_suffix
        self.job_title = job_title
        self.member_of = member_of
        self.telephone_numbers = telephone_numbers
