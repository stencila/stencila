# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .grant import Grant
ImageObject = ForwardRef("ImageObject")
from .person_or_organization import PersonOrOrganization
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
class MonetaryGrant(Grant):
    """
    A monetary grant.
    """

    type: Literal["MonetaryGrant"] = field(default="MonetaryGrant", init=False)

    amounts: Optional[float] = None
    """The amount of money."""

    funders: Optional[List[PersonOrOrganization]] = None
    """A person or organization that supports (sponsors) something through some kind of financial contribution."""

    def __init__(self, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, funded_items: Optional[List[Thing]] = None, sponsors: Optional[List[PersonOrOrganization]] = None, amounts: Optional[float] = None, funders: Optional[List[PersonOrOrganization]] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url, funded_items = funded_items, sponsors = sponsors)
        self.amounts = amounts
        self.funders = funders
