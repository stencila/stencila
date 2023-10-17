# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .code_error import CodeError
from .duration import Duration
from .entity import Entity
from .execution_auto import ExecutionAuto
from .execution_dependant import ExecutionDependant
from .execution_dependency import ExecutionDependency
from .execution_digest import ExecutionDigest
from .execution_required import ExecutionRequired
from .execution_status import ExecutionStatus
from .execution_tag import ExecutionTag
from .timestamp import Timestamp


@dataclass(kw_only=True, frozen=True)
class Executable(Entity):
    """
    Abstract base type for executable nodes (e.g. `CodeChunk`, `CodeExpression`, `Call`).
    """

    type: Literal["Executable"] = field(default="Executable", init=False)

    execution_auto: Optional[ExecutionAuto] = None
    """Under which circumstances the code should be automatically executed."""

    compilation_digest: Optional[ExecutionDigest] = None
    """A digest of the content, semantics and dependencies of the node."""

    execution_digest: Optional[ExecutionDigest] = None
    """The `compileDigest` of the node when it was last executed."""

    execution_dependencies: Optional[List[ExecutionDependency]] = None
    """The upstream dependencies of this node."""

    execution_dependants: Optional[List[ExecutionDependant]] = None
    """The downstream dependants of this node."""

    execution_tags: Optional[List[ExecutionTag]] = None
    """Tags in the code which affect its execution."""

    execution_count: Optional[int] = None
    """A count of the number of times that the node has been executed."""

    execution_required: Optional[ExecutionRequired] = None
    """Whether, and why, the code requires execution or re-execution."""

    execution_kernel: Optional[str] = None
    """The id of the kernel that the node was last executed in."""

    execution_status: Optional[ExecutionStatus] = None
    """Status of the most recent, including any current, execution."""

    execution_ended: Optional[Timestamp] = None
    """The timestamp when the last execution ended."""

    execution_duration: Optional[Duration] = None
    """Duration of the last execution."""

    errors: Optional[List[CodeError]] = None
    """Errors when compiling (e.g. syntax errors) or executing the node."""
