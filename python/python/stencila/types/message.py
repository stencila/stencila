# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .entity import Entity
from .message_part import MessagePart
from .person_or_organization_or_software_application import PersonOrOrganizationOrSoftwareApplication


@dataclass(init=False)
class Message(Entity):
    """
    A message from a sender to one or more people, organizations or software application.
    """

    type: Literal["Message"] = field(default="Message", init=False)

    parts: List[MessagePart]
    """Parts of the message."""

    content: Optional[List[Block]] = None
    """Content of the message."""

    authors: Optional[List[PersonOrOrganizationOrSoftwareApplication]] = None
    """The authors of the message."""

    def __init__(self, parts: List[MessagePart], id: Optional[str] = None, content: Optional[List[Block]] = None, authors: Optional[List[PersonOrOrganizationOrSoftwareApplication]] = None):
        super().__init__(id = id)
        self.parts = parts
        self.content = content
        self.authors = authors
