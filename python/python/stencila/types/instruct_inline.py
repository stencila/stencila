# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .execution_status import ExecutionStatus
from .inline import Inline
from .instruct import Instruct
from .person_or_organization_or_software_application import PersonOrOrganizationOrSoftwareApplication


@dataclass(init=False)
class InstructInline(Instruct):
    """
    An instruction to edit some inline content.
    """

    type: Literal["InstructInline"] = field(default="InstructInline", init=False)

    content: Optional[List[Inline]] = None
    """The content to which the instruction applies."""

    def __init__(self, text: str, id: Optional[str] = None, agent: Optional[PersonOrOrganizationOrSoftwareApplication] = None, execution_status: Optional[ExecutionStatus] = None, content: Optional[List[Inline]] = None):
        super().__init__(id = id, text = text, agent = agent, execution_status = execution_status)
        self.content = content
