# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(kw_only=True, frozen=True)
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
