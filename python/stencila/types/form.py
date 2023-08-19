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
from .form_derive_action import FormDeriveAction
from .int_or_str import IntOrStr
from .timestamp import Timestamp


class Form(BaseModel):
    """
    A form to batch updates in document parameters
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

    content: List[Block]
    """The content within the form, usually containing at least one `Parameter`."""

    derive_from: Optional[str]
    """The dotted path to the object (e.g a database table) that the form should be derived from"""

    derive_action: Optional[FormDeriveAction]
    """The action (create, update or delete) to derive for the form"""

    derive_item: Optional[IntOrStr]
    """An identifier for the item to be the target of Update or Delete actions"""
