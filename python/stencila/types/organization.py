# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .brand import Brand
from .contact_point import ContactPoint
from .image_object_or_str import ImageObjectOrStr
from .organization_or_person import OrganizationOrPerson
from .postal_address_or_str import PostalAddressOrStr
from .property_value_or_str import PropertyValueOrStr


class Organization(BaseModel):
    """
    An organization such as a school, NGO, corporation, club, etc.
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
    """Postal address for the organization."""

    brands: Optional[List[Brand]]
    """Brands that the organization is connected with."""

    contact_points: Optional[List[ContactPoint]]
    """Correspondence/Contact points for the organization."""

    departments: Optional[List[Organization]]
    """Departments within the organization. For example, Department of Computer Science, Research & Development etc."""

    funders: Optional[List[OrganizationOrPerson]]
    """Organization(s) or person(s) funding the organization."""

    legal_name: Optional[str]
    """The official name of the organization, e.g. the registered company name."""

    logo: Optional[ImageObjectOrStr]
    """The logo of the organization."""

    members: Optional[List[OrganizationOrPerson]]
    """Person(s) or organization(s) who are members of this organization."""

    parent_organization: Optional[Organization]
    """Entity that the Organization is a part of. For example, parentOrganization to a department is a university."""
