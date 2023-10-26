# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ImageObject = ForwardRef("ImageObject")
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
class DefinedTerm(Thing):
    """
    A word, name, acronym, phrase, etc. with a formal definition.
    """

    type: Literal["DefinedTerm"] = field(default="DefinedTerm", init=False)

    term_code: Optional[str] = None
    """A code that identifies this DefinedTerm within a DefinedTermSet"""

    def __init__(self, name: str, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, url: Optional[str] = None, term_code: Optional[str] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.term_code = term_code
