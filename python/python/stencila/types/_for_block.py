# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._array import Array
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
from ._execution_error import ExecutionError
from ._execution_required import ExecutionRequired
from ._execution_status import ExecutionStatus
from ._execution_tag import ExecutionTag
from ._timestamp import Timestamp


@dataclass(init=False)
class ForBlock(CodeExecutable):
    """
    Repeat a block content for each item in an array.
    """

    type: Literal["ForBlock"] = field(default="ForBlock", init=False)

    symbol: str
    """The name to give to the variable representing each item in the iterated array"""

    content: List[Block]
    """The content to repeat for each item"""

    otherwise: Optional[List[Block]] = None
    """The content to render if there are no items"""

    iterations: Optional[List[Array]] = None
    """The content repeated for each iteration"""

    def __init__(self, code: Cord, symbol: str, content: List[Block], id: Optional[str] = None, auto_exec: Optional[AutomaticExecution] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, execution_digest: Optional[CompilationDigest] = None, execution_dependencies: Optional[List[ExecutionDependency]] = None, execution_dependants: Optional[List[ExecutionDependant]] = None, execution_tags: Optional[List[ExecutionTag]] = None, execution_count: Optional[int] = None, execution_required: Optional[ExecutionRequired] = None, execution_status: Optional[ExecutionStatus] = None, execution_actor: Optional[str] = None, execution_ended: Optional[Timestamp] = None, execution_duration: Optional[Duration] = None, execution_errors: Optional[List[ExecutionError]] = None, programming_language: Optional[str] = None, authors: Optional[List[Author]] = None, otherwise: Optional[List[Block]] = None, iterations: Optional[List[Array]] = None):
        super().__init__(id = id, auto_exec = auto_exec, compilation_digest = compilation_digest, compilation_errors = compilation_errors, execution_digest = execution_digest, execution_dependencies = execution_dependencies, execution_dependants = execution_dependants, execution_tags = execution_tags, execution_count = execution_count, execution_required = execution_required, execution_status = execution_status, execution_actor = execution_actor, execution_ended = execution_ended, execution_duration = execution_duration, execution_errors = execution_errors, code = code, programming_language = programming_language, authors = authors)
        self.symbol = symbol
        self.content = content
        self.otherwise = otherwise
        self.iterations = iterations
