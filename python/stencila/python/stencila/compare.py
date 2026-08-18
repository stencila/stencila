"""Compare two documents semantically, rather than as text.

Neither side is presumed correct: they are the left and right snapshots that the caller
selected. Each pair of functions covers one kind of input -- in-memory nodes, strings,
or file paths -- and differs only in whether it returns the comparison artifact or a
human-readable rendering of it.

Comparison is CPU-bound and synchronous, so unlike `stencila.convert` none of these
functions need to be awaited.
"""

from __future__ import annotations

import json
from collections.abc import Sequence
from os import PathLike
from typing import Any, Literal, TypeAlias

from stencila_types.types import Node
from stencila_types.utilities import to_json

from stencila import _stencila

Comparison: TypeAlias = dict[str, Any]
Report: TypeAlias = Literal["text", "html"]


class CompareError(RuntimeError):
    """Raised when two documents cannot be compared."""


def compare_nodes(
    left: Node,
    right: Node,
    *,
    include: Sequence[str] = (),
    exclude: Sequence[str] = (),
    alignment_cell_budget: int | None = None,
) -> Comparison:
    """
    Compare two Stencila Schema nodes.

    Args:
        left (Node): The left node.
        right (Node): The right node.
        include (Sequence[str]): Selectors for differences to report. Only useful
            alongside a broader `exclude`, because with nothing excluded every
            difference is already reported.
        exclude (Sequence[str]): Selectors for differences not to report. A selector is
            a property (`jatsRefType`), a property of one node type (`Link.id`), a node
            type (`Link`), or `*` (or `all`) for everything. The most specific matching
            selector wins, whatever order they are given in.
        alignment_cell_budget (Optional[int]): The maximum number of candidate cells
            that sequence alignment may use.

    Returns:
        Comparison: The comparison, as a dict.
    """
    return _comparison(
        _stencila.compare.nodes,
        to_json(left),
        to_json(right),
        _options(include, exclude, alignment_cell_budget),
    )


def compare_strings(  # noqa: PLR0913
    left: str,
    right: str,
    *,
    left_format: str | None = None,
    right_format: str | None = None,
    include: Sequence[str] = (),
    exclude: Sequence[str] = (),
    alignment_cell_budget: int | None = None,
) -> Comparison:
    """
    Compare two documents in strings.

    Args:
        left (str): The left document.
        right (str): The right document.
        left_format (Optional[str]): The format of the left document. Defaults to
            JSON.
        right_format (Optional[str]): The format of the right document. Defaults to
            JSON.

    Returns:
        Comparison: The comparison, as a dict.

    See `compare_nodes` for the remaining arguments.
    """
    return _comparison(
        _stencila.compare.strings,
        left,
        right,
        _options(
            include,
            exclude,
            alignment_cell_budget,
            left_format=left_format,
            right_format=right_format,
        ),
    )


def compare_paths(  # noqa: PLR0913
    left: str | PathLike[str],
    right: str | PathLike[str],
    *,
    left_from: str | None = None,
    right_from: str | None = None,
    include: Sequence[str] = (),
    exclude: Sequence[str] = (),
    alignment_cell_budget: int | None = None,
) -> Comparison:
    """
    Compare two documents at filesystem paths.

    The two documents may be in different formats.

    Args:
        left (str | PathLike): The path of the left document.
        right (str | PathLike): The path of the right document.
        left_from (Optional[str]): The format of the left document. If not supplied, it
            is inferred from the path.
        right_from (Optional[str]): The format of the right document. If not supplied,
            it is inferred from the path.

    Returns:
        Comparison: The comparison, as a dict.

    See `compare_nodes` for the remaining arguments.
    """
    return _comparison(
        _stencila.compare.paths,
        str(left),
        str(right),
        _options(
            include,
            exclude,
            alignment_cell_budget,
            left_format=left_from,
            right_format=right_from,
        ),
    )


def report_nodes(  # noqa: PLR0913
    left: Node,
    right: Node,
    *,
    format: Report = "text",
    summary: bool = False,
    left_label: str = "left",
    right_label: str = "right",
    include: Sequence[str] = (),
    exclude: Sequence[str] = (),
    alignment_cell_budget: int | None = None,
) -> str:
    """
    Render a human-readable report of a comparison of two Stencila Schema nodes.

    Args:
        format (Report): "text" for a terminal report, or "html" for a self-contained
            side-by-side page. Defaults to "text".
        summary (bool): Whether to report only counts, not individual differences.
        left_label (str): What to call the left side in the report.
        right_label (str): What to call the right side in the report.

    Returns:
        str: The report.

    See `compare_nodes` for the remaining arguments.
    """
    return _report(
        _stencila.compare.nodes,
        to_json(left),
        to_json(right),
        _options(
            include,
            exclude,
            alignment_cell_budget,
            reports=[_format(format)],
            summary=summary,
            left_label=left_label,
            right_label=right_label,
        ),
        format,
    )


def report_strings(  # noqa: PLR0913
    left: str,
    right: str,
    *,
    format: Report = "text",
    summary: bool = False,
    left_format: str | None = None,
    right_format: str | None = None,
    left_label: str = "left",
    right_label: str = "right",
    include: Sequence[str] = (),
    exclude: Sequence[str] = (),
    alignment_cell_budget: int | None = None,
) -> str:
    """
    Render a human-readable report of a comparison of two documents in strings.

    Returns:
        str: The report.

    See `compare_strings` and `report_nodes` for the arguments.
    """
    return _report(
        _stencila.compare.strings,
        left,
        right,
        _options(
            include,
            exclude,
            alignment_cell_budget,
            left_format=left_format,
            right_format=right_format,
            reports=[_format(format)],
            summary=summary,
            left_label=left_label,
            right_label=right_label,
        ),
        format,
    )


def report_paths(  # noqa: PLR0913
    left: str | PathLike[str],
    right: str | PathLike[str],
    *,
    format: Report = "text",
    summary: bool = False,
    left_from: str | None = None,
    right_from: str | None = None,
    include: Sequence[str] = (),
    exclude: Sequence[str] = (),
    alignment_cell_budget: int | None = None,
) -> str:
    """
    Render a human-readable report of a comparison of two documents at paths.

    Each side is labelled with its path.

    Returns:
        str: The report.

    See `compare_paths` and `report_nodes` for the arguments.
    """
    return _report(
        _stencila.compare.paths,
        str(left),
        str(right),
        _options(
            include,
            exclude,
            alignment_cell_budget,
            left_format=left_from,
            right_format=right_from,
            reports=[_format(format)],
            summary=summary,
        ),
        format,
    )


def is_equal(comparison: Comparison) -> bool:
    """
    Whether a comparison found the two documents equal.

    Not simply an empty `differences`: a document with a whole subtree that the other
    lacks has no difference to report about it, only a one-sided correspondence. The
    question is answered by the comparison crate, so it means the same thing here as it
    does to the `stencila compare` exit code.

    Args:
        comparison (Comparison): A comparison, as returned by any of the `compare_`
            functions or read back from a comparison file.

    Returns:
        bool: Whether the two documents were equal, under whatever filter the comparison
            was made with.
    """
    return _stencila.compare.is_equal(json.dumps(comparison))


def _options(
    include: Sequence[str],
    exclude: Sequence[str],
    alignment_cell_budget: int | None,
    **rest: Any,
) -> dict[str, Any]:
    """Assemble the option dict that the native functions take.

    Every key is always present, because the native side reads them positionally out of
    the mapping rather than treating a missing key as unset.
    """
    return {
        "alignment_cell_budget": alignment_cell_budget,
        "include": list(include),
        "exclude": list(exclude),
        "left_format": None,
        "right_format": None,
        "left_label": None,
        "right_label": None,
        "reports": None,
        "summary": None,
        **rest,
    }


def _format(format: Report) -> str:
    """Reject an unknown report format before any document is decoded."""
    if format not in ("text", "html"):
        msg = "format must be 'text' or 'html'"
        raise ValueError(msg)
    return format


def _call(
    native: Any, left: str, right: str, options: dict[str, Any]
) -> dict[str, Any]:
    """Call a native comparison, reporting failures as `CompareError`.

    A `ValueError` is left alone: it means the caller supplied something unusable, such
    as a selector that is not in the schema, rather than that comparison failed.
    """
    try:
        return json.loads(native(left, right, options))
    except RuntimeError as error:
        raise CompareError(str(error)) from error


def _comparison(
    native: Any, left: str, right: str, options: dict[str, Any]
) -> Comparison:
    """Call a native comparison and return just the artifact."""
    return _call(native, left, right, options)["comparison"]


def _report(
    native: Any, left: str, right: str, options: dict[str, Any], format: Report
) -> str:
    """Call a native comparison and return just the rendering that was asked for."""
    return _call(native, left, right, options)[format]
