# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

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

    sender: Optional[PersonOrOrganizationOrSoftwareApplication] = None
    """The sender of the message."""

    def __init__(self, parts: List[MessagePart], id: Optional[str] = None, sender: Optional[PersonOrOrganizationOrSoftwareApplication] = None):
        super().__init__(id = id)
        self.parts = parts
        self.sender = sender
