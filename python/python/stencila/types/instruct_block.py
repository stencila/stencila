# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .execution_status import ExecutionStatus
from .instruct import Instruct
from .person_or_organization_or_software_application import PersonOrOrganizationOrSoftwareApplication


@dataclass(init=False)
class InstructBlock(Instruct):
    """
    An instruction to edit some block content.
    """

    type: Literal["InstructBlock"] = field(default="InstructBlock", init=False)

    content: Optional[List[Block]] = None
    """The content to which the instruction applies."""

    def __init__(self, text: str, id: Optional[str] = None, agent: Optional[PersonOrOrganizationOrSoftwareApplication] = None, execution_status: Optional[ExecutionStatus] = None, content: Optional[List[Block]] = None):
        super().__init__(id = id, text = text, agent = agent, execution_status = execution_status)
        self.content = content
