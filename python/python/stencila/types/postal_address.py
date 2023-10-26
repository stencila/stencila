# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .contact_point import ContactPoint
ImageObject = ForwardRef("ImageObject")
from .property_value_or_str import PropertyValueOrStr
from .text import Text


@dataclass(init=False)
class PostalAddress(ContactPoint):
    """
    A physical mailing address.
    """

    type: Literal["PostalAddress"] = field(default="PostalAddress", init=False)

    street_address: Optional[str] = None
    """The street address."""

    post_office_box_number: Optional[str] = None
    """The post office box number."""

    address_locality: Optional[str] = None
    """The locality in which the street address is, and which is in the region."""

    address_region: Optional[str] = None
    """The region in which the locality is, and which is in the country."""

    postal_code: Optional[str] = None
    """The postal code."""

    address_country: Optional[str] = None
    """The country."""

    def __init__(self, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, emails: Optional[List[str]] = None, telephone_numbers: Optional[List[str]] = None, available_languages: Optional[List[str]] = None, street_address: Optional[str] = None, post_office_box_number: Optional[str] = None, address_locality: Optional[str] = None, address_region: Optional[str] = None, postal_code: Optional[str] = None, address_country: Optional[str] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url, emails = emails, telephone_numbers = telephone_numbers, available_languages = available_languages)
        self.street_address = street_address
        self.post_office_box_number = post_office_box_number
        self.address_locality = address_locality
        self.address_region = address_region
        self.postal_code = postal_code
        self.address_country = address_country
