# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._automatic_execution import AutomaticExecution
from ._compilation_digest import CompilationDigest
from ._compilation_error import CompilationError
from ._cord import Cord
from ._duration import Duration
from ._execution_dependant import ExecutionDependant
from ._execution_dependency import ExecutionDependency
from ._execution_message import ExecutionMessage
from ._execution_required import ExecutionRequired
from ._execution_status import ExecutionStatus
from ._execution_tag import ExecutionTag
from ._node import Node
from ._parameter import Parameter
from ._timestamp import Timestamp
Validator = ForwardRef("Validator")


@dataclass(init=False)
class CallArgument(Parameter):
    """
    The value of a `Parameter` to call a document with.
    """

    type: Literal["CallArgument"] = field(default="CallArgument", init=False)

    code: Cord
    """The code to be evaluated for the parameter."""

    programming_language: Optional[str] = None
    """The programming language of the code."""

    def __init__(self, name: str, code: Cord, id: Optional[str] = None, auto_exec: Optional[AutomaticExecution] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, execution_digest: Optional[CompilationDigest] = None, execution_dependencies: Optional[List[ExecutionDependency]] = None, execution_dependants: Optional[List[ExecutionDependant]] = None, execution_tags: Optional[List[ExecutionTag]] = None, execution_count: Optional[int] = None, execution_required: Optional[ExecutionRequired] = None, execution_status: Optional[ExecutionStatus] = None, execution_actor: Optional[str] = None, execution_ended: Optional[Timestamp] = None, execution_duration: Optional[Duration] = None, execution_messages: Optional[List[ExecutionMessage]] = None, label: Optional[str] = None, value: Optional[Node] = None, default: Optional[Node] = None, validator: Optional[Validator] = None, derived_from: Optional[str] = None, programming_language: Optional[str] = None):
        super().__init__(id = id, auto_exec = auto_exec, compilation_digest = compilation_digest, compilation_errors = compilation_errors, execution_digest = execution_digest, execution_dependencies = execution_dependencies, execution_dependants = execution_dependants, execution_tags = execution_tags, execution_count = execution_count, execution_required = execution_required, execution_status = execution_status, execution_actor = execution_actor, execution_ended = execution_ended, execution_duration = execution_duration, execution_messages = execution_messages, name = name, label = label, value = value, default = default, validator = validator, derived_from = derived_from)
        self.code = code
        self.programming_language = programming_language
