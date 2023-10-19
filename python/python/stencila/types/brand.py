# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ImageObject = ForwardRef("ImageObject")
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class Brand(Thing):
    """
    A brand used by an organization or person for labeling a product, product group, or similar.
    """

    type: Literal["Brand"] = field(default="Brand", init=False)

    logo: Optional[ImageObject] = None
    """A logo associated with the brand."""

    reviews: Optional[List[str]] = None
    """Reviews of the brand."""
