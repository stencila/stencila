# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .contact_point import ContactPoint


@dataclass(kw_only=True, frozen=True)
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
