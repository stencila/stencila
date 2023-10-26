# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ImageObject = ForwardRef("ImageObject")
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
class Brand(Thing):
    """
    A brand used by an organization or person for labeling a product, product group, or similar.
    """

    type: Literal["Brand"] = field(default="Brand", init=False)

    logo: Optional[ImageObject] = None
    """A logo associated with the brand."""

    reviews: Optional[List[str]] = None
    """Reviews of the brand."""

    def __init__(self, name: str, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, url: Optional[str] = None, logo: Optional[ImageObject] = None, reviews: Optional[List[str]] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.logo = logo
        self.reviews = reviews
