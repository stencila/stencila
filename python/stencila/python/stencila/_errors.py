"""Give every failure this package raises one type to catch.

The native layer distinguishes only between a caller mistake (`ValueError`) and a
failure while doing the work (`RuntimeError`). Each public module refines the latter
into its own error so a caller can tell a comparison failure from a signing one, but
they share a base so that code which only wants to know that Stencila failed does not
have to name all of them.
"""

from __future__ import annotations


class StencilaError(RuntimeError):
    """Base class for failures raised while performing a Stencila operation.

    Deliberately not raised for a caller mistake: an unusable argument remains a
    `ValueError` (or `TypeError`), because it says something about the call rather
    than about the operation.
    """
