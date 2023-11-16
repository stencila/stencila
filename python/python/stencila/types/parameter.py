# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .automatic_execution import AutomaticExecution
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
from .node import Node
from .timestamp import Timestamp
Validator = ForwardRef("Validator")


@dataclass(init=False)
class Parameter(Executable):
    """
    A parameter of a document.
    """

    type: Literal["Parameter"] = field(default="Parameter", init=False)

    name: str
    """The name of the parameter."""

    label: Optional[str] = None
    """A short label for the parameter."""

    value: Optional[Node] = None
    """The current value of the parameter."""

    default: Optional[Node] = None
    """The default value of the parameter."""

    validator: Optional[Validator] = None
    """The validator that the value is validated against."""

    hidden: Optional[bool] = None
    """Whether the parameter should be hidden."""

    derived_from: Optional[str] = None
    """The dotted path to the object (e.g. a database table column) that the parameter should be derived from"""

    def __init__(self, name: str, id: Optional[str] = None, auto_exec: Optional[AutomaticExecution] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, execution_digest: Optional[CompilationDigest] = None, execution_dependencies: Optional[List[ExecutionDependency]] = None, execution_dependants: Optional[List[ExecutionDependant]] = None, execution_tags: Optional[List[ExecutionTag]] = None, execution_count: Optional[int] = None, execution_required: Optional[ExecutionRequired] = None, execution_kernel: Optional[str] = None, execution_status: Optional[ExecutionStatus] = None, execution_ended: Optional[Timestamp] = None, execution_duration: Optional[Duration] = None, execution_errors: Optional[List[ExecutionError]] = None, label: Optional[str] = None, value: Optional[Node] = None, default: Optional[Node] = None, validator: Optional[Validator] = None, hidden: Optional[bool] = None, derived_from: Optional[str] = None):
        super().__init__(id = id, auto_exec = auto_exec, compilation_digest = compilation_digest, compilation_errors = compilation_errors, execution_digest = execution_digest, execution_dependencies = execution_dependencies, execution_dependants = execution_dependants, execution_tags = execution_tags, execution_count = execution_count, execution_required = execution_required, execution_kernel = execution_kernel, execution_status = execution_status, execution_ended = execution_ended, execution_duration = execution_duration, execution_errors = execution_errors)
        self.name = name
        self.label = label
        self.value = value
        self.default = default
        self.validator = validator
        self.hidden = hidden
        self.derived_from = derived_from
