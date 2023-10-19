# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .brand import Brand
from .contact_point import ContactPoint
ImageObject = ForwardRef("ImageObject")
Organization = ForwardRef("Organization")
from .organization_or_person import OrganizationOrPerson
from .postal_address_or_str import PostalAddressOrStr
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class Organization(Thing):
    """
    An organization such as a school, NGO, corporation, club, etc.
    """

    type: Literal["Organization"] = field(default="Organization", init=False)

    address: Optional[PostalAddressOrStr] = None
    """Postal address for the organization."""

    brands: Optional[List[Brand]] = None
    """Brands that the organization is connected with."""

    contact_points: Optional[List[ContactPoint]] = None
    """Correspondence/Contact points for the organization."""

    departments: Optional[List[Organization]] = None
    """Departments within the organization. For example, Department of Computer Science, Research & Development etc."""

    funders: Optional[List[OrganizationOrPerson]] = None
    """Organization(s) or person(s) funding the organization."""

    legal_name: Optional[str] = None
    """The official name of the organization, e.g. the registered company name."""

    logo: Optional[ImageObject] = None
    """The logo of the organization."""

    members: Optional[List[OrganizationOrPerson]] = None
    """Person(s) or organization(s) who are members of this organization."""

    parent_organization: Optional[Organization] = None
    """Entity that the Organization is a part of. For example, parentOrganization to a department is a university."""
