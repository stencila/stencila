# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ImageObject = ForwardRef("ImageObject")
from .person_or_organization import PersonOrOrganization
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
class Grant(Thing):
    """
    A grant, typically financial or otherwise quantifiable, of resources.
    """

    type: Literal["Grant"] = field(default="Grant", init=False)

    funded_items: Optional[List[Thing]] = None
    """Indicates an item funded or sponsored through a Grant."""

    sponsors: Optional[List[PersonOrOrganization]] = None
    """A person or organization that supports a thing through a pledge, promise, or financial contribution."""

    def __init__(self, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, funded_items: Optional[List[Thing]] = None, sponsors: Optional[List[PersonOrOrganization]] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.funded_items = funded_items
        self.sponsors = sponsors
