"""Render common plot objects without importing optional plotting packages.

Runtime type inspection keeps plotting libraries optional while a small
registry provides an extension point for other object models.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping
from pathlib import Path
from types import ModuleType
from typing import Any

Renderer = Callable[[object, Path, Mapping[str, Any]], None]

_renderers: dict[type, Renderer] = {}


class UnsupportedPlotError(TypeError):
    """Raised when no plot renderer supports an object."""


def register_renderer(plot_type: type, renderer: Renderer) -> None:
    """Register a process-local renderer for a plot class.

    Exact types take precedence and base classes are considered in method
    resolution order, allowing integrations to choose their desired scope.
    """
    if not isinstance(plot_type, type):
        msg = "plot_type must be a type"
        raise TypeError(msg)
    if not callable(renderer):
        msg = "renderer must be callable"
        raise TypeError(msg)
    _renderers[plot_type] = renderer


def render(subject: object, path: Path, options: Mapping[str, Any]) -> None:
    """Render a supported live plot to a closed temporary path.

    Registered renderers run before built-in structural checks so applications
    can override behavior without requiring imports in this module.
    """
    subject_type = type(subject)
    for candidate in subject_type.__mro__:
        if renderer := _renderers.get(candidate):
            renderer(subject, path, options)
            return

    if _derives_from_matplotlib(subject_type, {"Figure"}):
        subject.savefig(path, **options)  # type: ignore[attr-defined]
        return
    if _derives_from_matplotlib(subject_type, {"Axes", "AxesSubplot"}):
        subject.figure.savefig(path, **options)  # type: ignore[attr-defined]
        return
    if (
        isinstance(subject, ModuleType)
        and subject.__name__ == "matplotlib.pyplot"
        and callable(getattr(subject, "savefig", None))
    ):
        subject.savefig(path, **options)  # type: ignore[attr-defined]
        return

    qualified = f"{subject_type.__module__}.{subject_type.__name__}"
    msg = (
        f"unsupported plot type {qualified}; built-in support covers Matplotlib "
        "Figure, Axes, and pyplot. Use stencila.credentials.register_renderer "
        "for other plot types."
    )
    raise UnsupportedPlotError(msg)


def _derives_from_matplotlib(subject_type: type, names: set[str]) -> bool:
    """Recognize a Matplotlib class by name without importing Matplotlib."""
    return any(
        base.__module__.startswith("matplotlib.") and base.__name__ in names
        for base in subject_type.__mro__
    )
