# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .automatic_execution import AutomaticExecution
from .block import Block
from .code_executable import CodeExecutable
from .compilation_digest import CompilationDigest
from .compilation_error import CompilationError
from .cord import Cord
from .duration import Duration
from .execution_dependant import ExecutionDependant
from .execution_dependency import ExecutionDependency
from .execution_error import ExecutionError
from .execution_required import ExecutionRequired
from .execution_status import ExecutionStatus
from .execution_tag import ExecutionTag
from .timestamp import Timestamp


@dataclass(init=False)
class IfBlockClause(CodeExecutable):
    """
    A clause within a `If` node.
    """

    type: Literal["IfBlockClause"] = field(default="IfBlockClause", init=False)

    is_active: Optional[bool] = None
    """Whether this clause is the active clause in the parent `If` node"""

    content: List[Block]
    """The content to render if the result is truthy"""

    def __init__(self, code: Cord, content: List[Block], id: Optional[str] = None, auto_exec: Optional[AutomaticExecution] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, execution_digest: Optional[CompilationDigest] = None, execution_dependencies: Optional[List[ExecutionDependency]] = None, execution_dependants: Optional[List[ExecutionDependant]] = None, execution_tags: Optional[List[ExecutionTag]] = None, execution_count: Optional[int] = None, execution_required: Optional[ExecutionRequired] = None, execution_kernel: Optional[str] = None, execution_status: Optional[ExecutionStatus] = None, execution_ended: Optional[Timestamp] = None, execution_duration: Optional[Duration] = None, execution_errors: Optional[List[ExecutionError]] = None, programming_language: Optional[str] = None, is_active: Optional[bool] = None):
        super().__init__(id = id, auto_exec = auto_exec, compilation_digest = compilation_digest, compilation_errors = compilation_errors, execution_digest = execution_digest, execution_dependencies = execution_dependencies, execution_dependants = execution_dependants, execution_tags = execution_tags, execution_count = execution_count, execution_required = execution_required, execution_kernel = execution_kernel, execution_status = execution_status, execution_ended = execution_ended, execution_duration = execution_duration, execution_errors = execution_errors, code = code, programming_language = programming_language)
        self.is_active = is_active
        self.content = content
