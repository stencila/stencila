# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .call_argument import CallArgument
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


class Call(BaseModel):
    """
    Call another document, optionally with arguments, and include its executed content.
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

    source: str
    """The external source of the content, a file path or URL."""

    media_type: Optional[str]
    """Media type of the source content."""

    select: Optional[str]
    """A query to select a subset of content from the source"""

    content: Optional[List[Block]]
    """The structured content decoded from the source."""

    arguments: List[CallArgument]
    """The value of the source document's parameters to call it with"""
