# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .brand import Brand
ImageObject = ForwardRef("ImageObject")
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class Product(Thing):
    """
    Any offered product or service. For example, a pair of shoes;    a haircut; or an episode of a TV show streamed online.
    """

    type: Literal["Product"] = field(default="Product", init=False)

    brands: Optional[List[Brand]] = None
    """Brands that the product is labelled with."""

    logo: Optional[ImageObject] = None
    """The logo of the product."""

    product_id: Optional[str] = None
    """Product identification code."""
