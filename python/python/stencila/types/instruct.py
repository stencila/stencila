# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .execution_status import ExecutionStatus
from .person_or_organization_or_software_application import PersonOrOrganizationOrSoftwareApplication


@dataclass(init=False)
class Instruct(Entity):
    """
    Abstract base type for a document editing instruction.
    """

    type: Literal["Instruct"] = field(default="Instruct", init=False)

    text: str
    """The text of the instruction."""

    agent: Optional[PersonOrOrganizationOrSoftwareApplication] = None
    """The agent that executed the instruction."""

    execution_status: Optional[ExecutionStatus] = None
    """Status of the execution of the instruction."""

    def __init__(self, text: str, id: Optional[str] = None, agent: Optional[PersonOrOrganizationOrSoftwareApplication] = None, execution_status: Optional[ExecutionStatus] = None):
        super().__init__(id = id)
        self.text = text
        self.agent = agent
        self.execution_status = execution_status
