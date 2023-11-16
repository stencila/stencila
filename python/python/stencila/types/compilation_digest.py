# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .unsigned_integer import UnsignedInteger


@dataclass(init=False)
class CompilationDigest(Entity):
    """
    A digest of the content, semantics and dependencies of an executable node.
    """

    type: Literal["CompilationDigest"] = field(default="CompilationDigest", init=False)

    state_digest: UnsignedInteger
    """A digest of the state of a node."""

    semantic_digest: Optional[UnsignedInteger] = None
    """A digest of the semantics of the node with respect to the dependency graph."""

    dependencies_digest: Optional[UnsignedInteger] = None
    """A digest of the semantic digests of the dependencies of a node."""

    dependencies_stale: Optional[UnsignedInteger] = None
    """A count of the number of dependencies that are stale."""

    dependencies_failed: Optional[UnsignedInteger] = None
    """A count of the number of dependencies that failed."""

    def __init__(self, state_digest: UnsignedInteger, id: Optional[str] = None, semantic_digest: Optional[UnsignedInteger] = None, dependencies_digest: Optional[UnsignedInteger] = None, dependencies_stale: Optional[UnsignedInteger] = None, dependencies_failed: Optional[UnsignedInteger] = None):
        super().__init__(id = id)
        self.state_digest = state_digest
        self.semantic_digest = semantic_digest
        self.dependencies_digest = dependencies_digest
        self.dependencies_stale = dependencies_stale
        self.dependencies_failed = dependencies_failed
