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
from .timestamp import Timestamp


@dataclass(init=False)
class IncludeBlock(Executable):
    """
    Include block content from an external source (e.g. file, URL).
    """

    type: Literal["IncludeBlock"] = field(default="IncludeBlock", init=False)

    source: str
    """The external source of the content, a file path or URL."""

    media_type: Optional[str] = None
    """Media type of the source content."""

    select: Optional[str] = None
    """A query to select a subset of content from the source"""

    content: Optional[List[Block]] = None
    """The structured content decoded from the source."""

    def __init__(self, source: str, id: Optional[str] = None, auto_exec: Optional[AutomaticExecution] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, execution_digest: Optional[CompilationDigest] = None, execution_dependencies: Optional[List[ExecutionDependency]] = None, execution_dependants: Optional[List[ExecutionDependant]] = None, execution_tags: Optional[List[ExecutionTag]] = None, execution_count: Optional[int] = None, execution_required: Optional[ExecutionRequired] = None, execution_kernel: Optional[str] = None, execution_status: Optional[ExecutionStatus] = None, execution_ended: Optional[Timestamp] = None, execution_duration: Optional[Duration] = None, execution_errors: Optional[List[ExecutionError]] = None, media_type: Optional[str] = None, select: Optional[str] = None, content: Optional[List[Block]] = None):
        super().__init__(id = id, auto_exec = auto_exec, compilation_digest = compilation_digest, compilation_errors = compilation_errors, execution_digest = execution_digest, execution_dependencies = execution_dependencies, execution_dependants = execution_dependants, execution_tags = execution_tags, execution_count = execution_count, execution_required = execution_required, execution_kernel = execution_kernel, execution_status = execution_status, execution_ended = execution_ended, execution_duration = execution_duration, execution_errors = execution_errors)
        self.source = source
        self.media_type = media_type
        self.select = select
        self.content = content
