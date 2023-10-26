# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(init=False)
class ExecutionDigest(Entity):
    """
    A digest of the execution state of a node.
    """

    type: Literal["ExecutionDigest"] = field(default="ExecutionDigest", init=False)

    state_digest: float
    """A digest of the state of a node."""

    semantic_digest: float
    """A digest of the "semantic intent" of the resource with respect to the dependency graph"""

    dependencies_digest: float
    """A digest of the semantic digests the dependencies of a resource."""

    dependencies_stale: float
    """A count of the number of execution dependencies that are stale"""

    dependencies_failed: float
    """A count of the number of execution dependencies that failed"""

    def __init__(self, state_digest: float, semantic_digest: float, dependencies_digest: float, dependencies_stale: float, dependencies_failed: float, id: Optional[str] = None):
        super().__init__(id = id)
        self.state_digest = state_digest
        self.semantic_digest = semantic_digest
        self.dependencies_digest = dependencies_digest
        self.dependencies_stale = dependencies_stale
        self.dependencies_failed = dependencies_failed
