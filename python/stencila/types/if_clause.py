# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .code_error import CodeError
from .duration import Duration
from .execution_auto import ExecutionAuto
from .execution_dependant import ExecutionDependant
from .execution_dependency import ExecutionDependency
from .execution_digest import ExecutionDigest
from .execution_required import ExecutionRequired
from .execution_status import ExecutionStatus
from .execution_tag import ExecutionTag
from .timestamp import Timestamp


class IfClause(BaseModel):
    """
    A clause within a `If` node
    """

    id: Optional[str]
    """The identifier for this item"""

    execution_auto: Optional[ExecutionAuto]
    """Under which circumstances the code should be automatically executed."""

    compilation_digest: Optional[ExecutionDigest]
    """A digest of the content, semantics and dependencies of the node."""

    execution_digest: Optional[ExecutionDigest]
    """The `compileDigest` of the node when it was last executed."""

    execution_dependencies: Optional[List[ExecutionDependency]]
    """The upstream dependencies of this node."""

    execution_dependants: Optional[List[ExecutionDependant]]
    """The downstream dependants of this node."""

    execution_tags: Optional[List[ExecutionTag]]
    """Tags in the code which affect its execution"""

    execution_count: Optional[int]
    """A count of the number of times that the node has been executed."""

    execution_required: Optional[ExecutionRequired]
    """Whether, and why, the code requires execution or re-execution."""

    execution_kernel: Optional[str]
    """The id of the kernel that the node was last executed in."""

    execution_status: Optional[ExecutionStatus]
    """Status of the most recent, including any current, execution."""

    execution_ended: Optional[Timestamp]
    """The timestamp when the last execution ended."""

    execution_duration: Optional[Duration]
    """Duration of the last execution."""

    errors: Optional[List[CodeError]]
    """Errors when compiling (e.g. syntax errors) or executing the node."""

    code: str
    """The code."""

    programming_language: str
    """The programming language of the code."""

    guess_language: Optional[bool]
    """Whether the programming language of the code should be guessed based on syntax and variables used"""

    media_type: Optional[str]
    """Media type, typically expressed using a MIME format, of the code."""

    is_active: Optional[bool]
    """Whether this clause is the active clause in the parent `If` node"""

    content: List[Block]
    """The content to render if the result is true-thy"""
