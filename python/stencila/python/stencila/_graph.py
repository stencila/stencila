"""Keep graph previews identical to the assertions used during signing.

Both public inspection and credential production pass through one preparation
path so rendering, context inference, and graph projection cannot drift.
"""

from __future__ import annotations

import json
import shutil
import tempfile
from collections.abc import Mapping
from os import PathLike
from pathlib import Path
from typing import Any, Literal, TypeAlias

from stencila import _stencila

from ._context import absolute, resolve_context
from ._errors import StencilaError
from ._render import render
from ._types import Graph, graph_from_data

Profile: TypeAlias = Literal["public", "private", "full"]
Provenance: TypeAlias = Literal["auto", "required", "none"]


class ProvenanceError(StencilaError):
    """Base error for provenance construction."""


class ProvenanceNotFoundError(ProvenanceError):
    """Raised when required source-linked provenance is unavailable."""


def graph(  # noqa: PLR0913
    subject: object | str | PathLike[str],
    *,
    output: str | PathLike[str] | None = None,
    source: str | PathLike[str] | None = None,
    workspace: str | PathLike[str] | None = None,
    profile: Profile = "public",
    provenance: Provenance = "auto",
    render_options: Mapping[str, Any] | None = None,
) -> Graph:
    """Preview the exact graph that signing the asset would embed.

    Preparation is shared with signing so this result is useful for reviewing
    disclosure and provenance policy before creating a credential.
    """
    prepared, temporary = _prepare(
        subject,
        output=output,
        source=source,
        workspace=workspace,
        profile=profile,
        provenance=provenance,
        render_options=render_options,
    )
    _discard(temporary)
    return graph_from_data(prepared["graph"])


def _prepare(  # noqa: PLR0913
    subject: object | str | PathLike[str],
    *,
    output: str | PathLike[str] | None,
    source: str | PathLike[str] | None,
    workspace: str | PathLike[str] | None,
    profile: Profile,
    provenance: Provenance,
    render_options: Mapping[str, Any] | None,
    title: str | None = None,
) -> tuple[dict[str, Any], Path | None]:
    """Prepare one graph payload for either inspection or signing.

    Inputs are staged once to temporary bytes, and ownership of those bytes is
    returned to the caller so preparation and signing use the same stable
    snapshot.
    """
    _validate_options(profile, provenance)
    if isinstance(subject, str | PathLike):
        subject_path = absolute(subject)
        if not subject_path.is_file():
            msg = f"asset does not exist: {subject_path}"
            raise FileNotFoundError(msg)
        planned = absolute(output) if output else subject_path
        context = resolve_context(
            planned,
            source=source,
            workspace=workspace,
            infer_source=False,
            asset=subject_path,
        )
        lookup_path = subject_path
        temporary = _temporary_path(planned)
        try:
            shutil.copy2(subject_path, temporary)
        except Exception:
            _discard(temporary)
            raise
        input_path = temporary
    else:
        if output is None:
            msg = "output is required for an in-memory plot"
            raise ValueError(msg)
        planned = absolute(output)
        context = resolve_context(
            planned,
            source=source,
            workspace=workspace,
            infer_source=True,
        )
        if provenance == "required" and context.source is None:
            msg = "source-linked provenance is required but source inference failed"
            raise ProvenanceNotFoundError(msg)
        lookup_path = planned
        temporary = _temporary_path(planned)
        try:
            render(subject, temporary, render_options or {})
        except Exception:
            _discard(temporary)
            raise
        input_path = temporary

    # Ownership of the staged bytes passes to the caller only on success, so any
    # failure here - not just the native errors below - discards them rather
    # than leaving them in the caller's output directory.
    handed_over = False
    try:
        raw = _stencila.graph.prepare(
            str(input_path),
            str(planned),
            str(lookup_path),
            str(context.workspace),
            str(context.source) if context.source else None,
            context.source_line,
            profile,
            provenance,
            title,
        )
        prepared: dict[str, Any] = json.loads(raw)
        prepared["warnings"] = [*context.warnings, *prepared["warnings"]]
        handed_over = True
        return prepared, temporary
    except RuntimeError as error:
        error_type = (
            ProvenanceNotFoundError if provenance == "required" else ProvenanceError
        )
        raise error_type(str(error)) from error
    finally:
        if not handed_over:
            _discard(temporary)


def _discard(temporary: Path | None) -> None:
    """Remove rendered bytes once their owner no longer needs them."""
    if temporary is not None:
        temporary.unlink(missing_ok=True)


def _temporary_path(output: Path) -> Path:
    """Reserve a closed temporary path with the final asset's suffix.

    Renderers often infer their format from the suffix and may need to reopen
    the path themselves, so the temporary file cannot remain open here.
    """
    directory = output.parent if output.parent.is_dir() else None
    with tempfile.NamedTemporaryFile(
        prefix=".stencila-unsigned-",
        suffix=output.suffix,
        dir=directory,
        delete=False,
    ) as handle:
        return Path(handle.name)


def _validate_options(profile: str, provenance: str) -> None:
    """Fail early when policy values cannot be interpreted by native code."""
    if profile not in {"public", "private", "full"}:
        msg = "profile must be 'public', 'private', or 'full'"
        raise ValueError(msg)
    if provenance not in {"auto", "required", "none"}:
        msg = "provenance must be 'auto', 'required', or 'none'"
        raise ValueError(msg)
