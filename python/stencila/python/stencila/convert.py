"""Encode and decode Stencila documents, and convert between formats.

Decoding and encoding are performed by the native codecs, which release the interpreter
lock while they work. These functions are synchronous, like the rest of the package: a
caller who is already inside an event loop should dispatch them with
`asyncio.to_thread` rather than calling them directly.
"""

from __future__ import annotations

from collections.abc import Callable
from os import PathLike
from typing import Any, TypeVar

from stencila_types.types import Node
from stencila_types.utilities import from_json, to_json

from stencila import _stencila

from ._errors import StencilaError

T = TypeVar("T")


class ConvertError(StencilaError):
    """Raised when a document cannot be decoded, encoded, or converted."""


def from_string(string: str, format: str | None = "json") -> Node:
    """
    Decode a Stencila Schema node from a string.

    Args:
        string (str): The string to decode to a node.
        format (Optional[str]): The format to decode from. Defaults to "json".

    Returns:
        Node: A Stencila Schema node.
    """
    return from_json(_call(_stencila.convert.from_string, string, format=format))


def from_path(path: str | PathLike[str], format: str | None = None) -> Node:
    """
    Decode a Stencila Schema node from a filesystem path.

    Args:
        path (str | PathLike): The path to decode to a node.
        format (Optional[str]): The format to decode from. If not supplied, it
            is inferred from the path.

    Returns:
        Node: A Stencila Schema node.
    """
    return from_json(_call(_stencila.convert.from_path, str(path), format=format))


def to_string(
    node: Node,
    *,
    format: str | None = "json",
    standalone: bool = False,
    compact: bool = False,
) -> str:
    """
    Encode a Stencila Schema node to a string.

    Args:
        node (Node): The node to encode.
        format (Optional[str]): The format to encode to. Defaults to "json".
        standalone (bool): Whether to encode as a standalone document. Defaults
            to False.
        compact (bool): Whether to encode in compact form. Defaults to False.

    Returns:
        str: The node encoded as a string in the specified format.
    """
    return _call(
        _stencila.convert.to_string,
        to_json(node),
        format=format,
        standalone=standalone,
        compact=compact,
    )


def to_path(
    node: Node,
    path: str | PathLike[str],
    *,
    format: str | None = None,
    standalone: bool = False,
    compact: bool = False,
) -> None:
    """
    Encode a Stencila Schema node to a filesystem path.

    Args:
        node (Node): The node to encode.
        path (str | PathLike): The path to encode the node to.
        format (Optional[str]): The format to encode to. If not supplied, it is
            inferred from the path.
        standalone (bool): Whether to encode as a standalone document. Defaults
            to False.
        compact (bool): Whether to encode in compact form. Defaults to False.
    """
    _call(
        _stencila.convert.to_path,
        to_json(node),
        str(path),
        format=format,
        standalone=standalone,
        compact=compact,
    )


def from_to(  # noqa: PLR0913
    input: str | PathLike[str] | None = None,
    output: str | PathLike[str] | None = None,
    *,
    from_format: str | None = None,
    to_format: str | None = None,
    to_standalone: bool = False,
    to_compact: bool = False,
) -> str:
    """
    Convert a document from one format to another.

    Args:
        input (Optional[str | PathLike]): The input path. If not supplied, stdin
            will be read.
        output (Optional[str | PathLike]): The output path. If not supplied, the
            converted input will be returned.
        from_format (Optional[str]): The format of the input. If not supplied,
            inferred from the input path.
        to_format (Optional[str]): The format of the output. If not supplied,
            inferred from the output path.
        to_standalone (bool): Whether to encode as a standalone document.
            Defaults to False.
        to_compact (bool): Whether to encode in compact form. Defaults to
            False.

    Returns:
        str: The converted document as a string, or the path to the converted document.
    """
    return _call(
        _stencila.convert.from_to,
        str(input) if input is not None else None,
        str(output) if output is not None else None,
        from_format=from_format,
        to_format=to_format,
        to_standalone=to_standalone,
        to_compact=to_compact,
    )


def _call(native: Callable[..., T], *args: Any, **kwargs: Any) -> T:
    """Call a native conversion, reporting failures as `ConvertError`.

    A `ValueError` is left alone: it means a node could not be serialized or
    deserialized, which says something about the argument rather than about the
    conversion. Note that an unknown format is not one of these: the codecs discover
    that no codec supports it while converting, so it arrives as a `ConvertError`.
    """
    try:
        return native(*args, **kwargs)
    except RuntimeError as error:
        raise ConvertError(str(error)) from error


__all__ = [
    "ConvertError",
    "from_path",
    "from_string",
    "from_to",
    "to_path",
    "to_string",
]
