# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .automatic_execution import AutomaticExecution
from .code_error import CodeError
from .code_executable import CodeExecutable
from .cord import Cord
from .duration import Duration
from .execution_dependant import ExecutionDependant
from .execution_dependency import ExecutionDependency
from .execution_digest import ExecutionDigest
from .execution_required import ExecutionRequired
from .execution_status import ExecutionStatus
from .execution_tag import ExecutionTag
from .node import Node
from .timestamp import Timestamp


@dataclass(init=False)
class CodeExpression(CodeExecutable):
    """
    An executable programming code expression.
    """

    type: Literal["CodeExpression"] = field(default="CodeExpression", init=False)

    output: Optional[Node] = None
    """The value of the expression when it was last evaluated."""

    def __init__(self, code: Cord, id: Optional[str] = None, auto_exec: Optional[AutomaticExecution] = None, compilation_digest: Optional[ExecutionDigest] = None, execution_digest: Optional[ExecutionDigest] = None, execution_dependencies: Optional[List[ExecutionDependency]] = None, execution_dependants: Optional[List[ExecutionDependant]] = None, execution_tags: Optional[List[ExecutionTag]] = None, execution_count: Optional[int] = None, execution_required: Optional[ExecutionRequired] = None, execution_kernel: Optional[str] = None, execution_status: Optional[ExecutionStatus] = None, execution_ended: Optional[Timestamp] = None, execution_duration: Optional[Duration] = None, errors: Optional[List[CodeError]] = None, programming_language: Optional[str] = None, output: Optional[Node] = None):
        super().__init__(id = id, auto_exec = auto_exec, compilation_digest = compilation_digest, execution_digest = execution_digest, execution_dependencies = execution_dependencies, execution_dependants = execution_dependants, execution_tags = execution_tags, execution_count = execution_count, execution_required = execution_required, execution_kernel = execution_kernel, execution_status = execution_status, execution_ended = execution_ended, execution_duration = execution_duration, errors = errors, code = code, programming_language = programming_language)
        self.output = output
