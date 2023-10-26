# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .brand import Brand
from .contact_point import ContactPoint
ImageObject = ForwardRef("ImageObject")
Organization = ForwardRef("Organization")
from .person_or_organization import PersonOrOrganization
from .postal_address_or_str import PostalAddressOrStr
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
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

    funders: Optional[List[PersonOrOrganization]] = None
    """Organization(s) or person(s) funding the organization."""

    legal_name: Optional[str] = None
    """The official name of the organization, e.g. the registered company name."""

    logo: Optional[ImageObject] = None
    """The logo of the organization."""

    members: Optional[List[PersonOrOrganization]] = None
    """Person(s) or organization(s) who are members of this organization."""

    parent_organization: Optional[Organization] = None
    """Entity that the Organization is a part of. For example, parentOrganization to a department is a university."""

    def __init__(self, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, address: Optional[PostalAddressOrStr] = None, brands: Optional[List[Brand]] = None, contact_points: Optional[List[ContactPoint]] = None, departments: Optional[List[Organization]] = None, funders: Optional[List[PersonOrOrganization]] = None, legal_name: Optional[str] = None, logo: Optional[ImageObject] = None, members: Optional[List[PersonOrOrganization]] = None, parent_organization: Optional[Organization] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.address = address
        self.brands = brands
        self.contact_points = contact_points
        self.departments = departments
        self.funders = funders
        self.legal_name = legal_name
        self.logo = logo
        self.members = members
        self.parent_organization = parent_organization
