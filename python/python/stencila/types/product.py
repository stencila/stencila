# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .brand import Brand
ImageObject = ForwardRef("ImageObject")
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
class Product(Thing):
    """
    Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
    """

    type: Literal["Product"] = field(default="Product", init=False)

    brands: Optional[List[Brand]] = None
    """Brands that the product is labelled with."""

    logo: Optional[ImageObject] = None
    """The logo of the product."""

    product_id: Optional[str] = None
    """Product identification code."""

    def __init__(self, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, brands: Optional[List[Brand]] = None, logo: Optional[ImageObject] = None, product_id: Optional[str] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.brands = brands
        self.logo = logo
        self.product_id = product_id
