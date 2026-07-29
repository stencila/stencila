"""Keep provenance selection context consistent across public operations.

Source inference is useful runtime context, but it is centralized here so it
cannot silently become stronger evidence than the graph builder can support.
"""

from __future__ import annotations

import inspect
from dataclasses import dataclass
from os import PathLike
from pathlib import Path
from typing import overload


@dataclass(frozen=True)
class Context:
    """Carry normalized paths and inference diagnostics into graph preparation.

    Resolving these values once prevents rendering, inspection, and signing
    from making different workspace or source choices.
    """

    source: Path | None
    source_line: int | None
    workspace: Path
    output: Path
    warnings: tuple[str, ...] = ()


def resolve_context(
    output: str | PathLike[str],
    *,
    source: str | PathLike[str] | None,
    workspace: str | PathLike[str] | None,
    infer_source: bool,
    asset: str | PathLike[str] | None = None,
) -> Context:
    """Resolve stable source, workspace, and output paths for one operation.

    Explicit values always win. Inference supplies convenient defaults while
    preserving a warning when caller source cannot be established.
    """
    output_path = absolute(output)
    source_path = absolute(source)
    asset_path = absolute(asset)
    source_line = None
    warnings: list[str] = []

    if source_path is None and infer_source:
        source_path, source_line = _caller_source()
        if source_path is None:
            warnings.append("source inference unavailable")

    workspace_path = absolute(workspace) or _infer_workspace(
        source_path, output_path, asset_path
    )
    return Context(
        source=source_path,
        source_line=source_line,
        workspace=workspace_path,
        output=output_path,
        warnings=tuple(warnings),
    )


@overload
def absolute(value: str | PathLike[str]) -> Path: ...


@overload
def absolute(value: None) -> None: ...


def absolute(value: str | PathLike[str] | None) -> Path | None:
    """Normalize an optional caller path without repeating the guard clause."""
    return Path(value).expanduser().absolute() if value is not None else None


def _caller_source() -> tuple[Path | None, int | None]:
    """Find the first external Python frame as runtime provenance context.

    Package and import machinery frames are excluded because they describe the
    credential implementation rather than the code that produced the asset.
    """
    package_dir = Path(__file__).resolve().parent
    # Context lines are never used, and collecting them for every frame is the
    # dominant cost of walking the stack.
    for frame in inspect.stack(0):
        filename = frame.filename
        if filename.startswith("<"):
            continue
        path = Path(filename)
        if not path.is_file():
            continue
        resolved = path.resolve()
        if resolved == package_dir or package_dir in resolved.parents:
            continue
        if "importlib" in resolved.parts:
            continue
        if resolved.suffix == ".py":
            return resolved, frame.lineno - 1
    return None, None


def _infer_workspace(source: Path | None, output: Path, asset: Path | None) -> Path:
    """Prefer the nearest marked project shared by source and signed input.

    Project markers keep provenance paths stable when a call originates below
    the workspace root. Existing assets anchor exports outside their project,
    while the planned output remains the anchor for live renders.

    Inference never widens beyond the asset's own directory. A source on an
    unrelated branch of the filesystem - a notebook cell under a temporary
    directory, say - would otherwise make the common ancestor `$HOME` or `/`,
    and the workspace scan that follows would walk all of it.
    """
    provenance_output = asset or output
    fallback = _directory(provenance_output)

    if source is not None:
        boundary = _common_ancestor([source, provenance_output])
        # Anchor discovery on one of the paths supplied for this operation.
        # Searching upward from their common ancestor can select an unrelated
        # marker at `$HOME`, `/tmp`, or another broad parent and scan far beyond
        # the asset's project.
        return (
            _nearest_marked_ancestor(provenance_output, boundary)
            or _nearest_marked_ancestor(source, boundary)
            or fallback
        )

    return _nearest_marked_ancestor(provenance_output) or fallback


def _directory(path: Path) -> Path:
    """Treat a path as its containing directory unless it is already one."""
    return path if path.is_dir() else path.parent


def _nearest_marked_ancestor(path: Path, boundary: Path | None = None) -> Path | None:
    """Find the nearest project marker without searching above a boundary."""
    directory = _directory(path)
    for candidate in (directory, *directory.parents):
        if any(
            (candidate / marker).exists()
            for marker in ("stencila.toml", "stencila.local.toml", ".git")
        ):
            return candidate
        if candidate == boundary:
            break
    return None


def _common_ancestor(paths: list[Path]) -> Path | None:
    """Find the nearest directory containing every supplied path."""
    if not paths:
        return None
    first, *rest = [_directory(path) for path in paths]
    for candidate in (first, *first.parents):
        if all(path == candidate or candidate in path.parents for path in rest):
            return candidate
    return None
