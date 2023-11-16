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
from .node import Node
from .timestamp import Timestamp


@dataclass(init=False)
class CodeChunk(CodeExecutable):
    """
    A executable chunk of code.
    """

    type: Literal["CodeChunk"] = field(default="CodeChunk", init=False)

    execution_pure: Optional[bool] = None
    """Whether the code should be treated as side-effect free when executed."""

    outputs: Optional[List[Node]] = None
    """Outputs from executing the chunk."""

    label: Optional[str] = None
    """A short label for the CodeChunk."""

    caption: Optional[List[Block]] = None
    """A caption for the CodeChunk."""

    def __init__(self, code: Cord, id: Optional[str] = None, auto_exec: Optional[AutomaticExecution] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, execution_digest: Optional[CompilationDigest] = None, execution_dependencies: Optional[List[ExecutionDependency]] = None, execution_dependants: Optional[List[ExecutionDependant]] = None, execution_tags: Optional[List[ExecutionTag]] = None, execution_count: Optional[int] = None, execution_required: Optional[ExecutionRequired] = None, execution_kernel: Optional[str] = None, execution_status: Optional[ExecutionStatus] = None, execution_ended: Optional[Timestamp] = None, execution_duration: Optional[Duration] = None, execution_errors: Optional[List[ExecutionError]] = None, programming_language: Optional[str] = None, execution_pure: Optional[bool] = None, outputs: Optional[List[Node]] = None, label: Optional[str] = None, caption: Optional[List[Block]] = None):
        super().__init__(id = id, auto_exec = auto_exec, compilation_digest = compilation_digest, compilation_errors = compilation_errors, execution_digest = execution_digest, execution_dependencies = execution_dependencies, execution_dependants = execution_dependants, execution_tags = execution_tags, execution_count = execution_count, execution_required = execution_required, execution_kernel = execution_kernel, execution_status = execution_status, execution_ended = execution_ended, execution_duration = execution_duration, execution_errors = execution_errors, code = code, programming_language = programming_language)
        self.execution_pure = execution_pure
        self.outputs = outputs
        self.label = label
        self.caption = caption
