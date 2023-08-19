# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ExecutionDigest(BaseModel):
    """
    A digest of the execution state of a node.
    """

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
