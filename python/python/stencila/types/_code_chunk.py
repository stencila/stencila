# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._author import Author
from ._automatic_execution import AutomaticExecution
from ._block import Block
from ._code_executable import CodeExecutable
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
from ._label_type import LabelType
from ._node import Node
from ._timestamp import Timestamp


@dataclass(init=False)
class CodeChunk(CodeExecutable):
    """
    A executable chunk of code.
    """

    type: Literal["CodeChunk"] = field(default="CodeChunk", init=False)

    label_type: Optional[LabelType] = None
    """The type of the label for the chunk."""

    label: Optional[str] = None
    """A short label for the chunk."""

    caption: Optional[List[Block]] = None
    """A caption for the chunk."""

    outputs: Optional[List[Node]] = None
    """Outputs from executing the chunk."""

    execution_pure: Optional[bool] = None
    """Whether the code should be treated as side-effect free when executed."""

    def __init__(self, code: Cord, id: Optional[str] = None, auto_exec: Optional[AutomaticExecution] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, execution_digest: Optional[CompilationDigest] = None, execution_dependencies: Optional[List[ExecutionDependency]] = None, execution_dependants: Optional[List[ExecutionDependant]] = None, execution_tags: Optional[List[ExecutionTag]] = None, execution_count: Optional[int] = None, execution_required: Optional[ExecutionRequired] = None, execution_status: Optional[ExecutionStatus] = None, execution_actor: Optional[str] = None, execution_ended: Optional[Timestamp] = None, execution_duration: Optional[Duration] = None, execution_messages: Optional[List[ExecutionMessage]] = None, programming_language: Optional[str] = None, authors: Optional[List[Author]] = None, label_type: Optional[LabelType] = None, label: Optional[str] = None, caption: Optional[List[Block]] = None, outputs: Optional[List[Node]] = None, execution_pure: Optional[bool] = None):
        super().__init__(id = id, auto_exec = auto_exec, compilation_digest = compilation_digest, compilation_errors = compilation_errors, execution_digest = execution_digest, execution_dependencies = execution_dependencies, execution_dependants = execution_dependants, execution_tags = execution_tags, execution_count = execution_count, execution_required = execution_required, execution_status = execution_status, execution_actor = execution_actor, execution_ended = execution_ended, execution_duration = execution_duration, execution_messages = execution_messages, code = code, programming_language = programming_language, authors = authors)
        self.label_type = label_type
        self.label = label
        self.caption = caption
        self.outputs = outputs
        self.execution_pure = execution_pure
