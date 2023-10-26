# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

ImageObject = ForwardRef("ImageObject")
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
class ContactPoint(Thing):
    """
    A contact point, usually within an organization.
    """

    type: Literal["ContactPoint"] = field(default="ContactPoint", init=False)

    emails: Optional[List[str]] = None
    """Email address for correspondence."""

    telephone_numbers: Optional[List[str]] = None
    """Telephone numbers for the contact point."""

    available_languages: Optional[List[str]] = None
    """Languages (human not programming) in which it is possible to communicate with the organization/department etc."""

    def __init__(self, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, emails: Optional[List[str]] = None, telephone_numbers: Optional[List[str]] = None, available_languages: Optional[List[str]] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.emails = emails
        self.telephone_numbers = telephone_numbers
        self.available_languages = available_languages
