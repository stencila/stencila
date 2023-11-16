# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .automatic_execution import AutomaticExecution
from .block import Block
from .compilation_digest import CompilationDigest
from .compilation_error import CompilationError
from .duration import Duration
from .executable import Executable
from .execution_dependant import ExecutionDependant
from .execution_dependency import ExecutionDependency
from .execution_error import ExecutionError
from .execution_required import ExecutionRequired
from .execution_status import ExecutionStatus
from .execution_tag import ExecutionTag
from .form_derive_action import FormDeriveAction
from .int_or_str import IntOrStr
from .timestamp import Timestamp


@dataclass(init=False)
class Form(Executable):
    """
    A form to batch updates in document parameters.
    """

    type: Literal["Form"] = field(default="Form", init=False)

    content: List[Block]
    """The content within the form, usually containing at least one `Parameter`."""

    derive_from: Optional[str] = None
    """The dotted path to the object (e.g a database table) that the form should be derived from"""

    derive_action: Optional[FormDeriveAction] = None
    """The action (create, update or delete) to derive for the form"""

    derive_item: Optional[IntOrStr] = None
    """An identifier for the item to be the target of Update or Delete actions"""

    def __init__(self, content: List[Block], id: Optional[str] = None, auto_exec: Optional[AutomaticExecution] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, execution_digest: Optional[CompilationDigest] = None, execution_dependencies: Optional[List[ExecutionDependency]] = None, execution_dependants: Optional[List[ExecutionDependant]] = None, execution_tags: Optional[List[ExecutionTag]] = None, execution_count: Optional[int] = None, execution_required: Optional[ExecutionRequired] = None, execution_kernel: Optional[str] = None, execution_status: Optional[ExecutionStatus] = None, execution_ended: Optional[Timestamp] = None, execution_duration: Optional[Duration] = None, execution_errors: Optional[List[ExecutionError]] = None, derive_from: Optional[str] = None, derive_action: Optional[FormDeriveAction] = None, derive_item: Optional[IntOrStr] = None):
        super().__init__(id = id, auto_exec = auto_exec, compilation_digest = compilation_digest, compilation_errors = compilation_errors, execution_digest = execution_digest, execution_dependencies = execution_dependencies, execution_dependants = execution_dependants, execution_tags = execution_tags, execution_count = execution_count, execution_required = execution_required, execution_kernel = execution_kernel, execution_status = execution_status, execution_ended = execution_ended, execution_duration = execution_duration, execution_errors = execution_errors)
        self.content = content
        self.derive_from = derive_from
        self.derive_action = derive_action
        self.derive_item = derive_item
