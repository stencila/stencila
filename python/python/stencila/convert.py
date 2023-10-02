from typing import Optional

from stencila import _stencila
from stencila.types import Node
from stencila.utilities import to_json, from_json


async def from_string(string: str, format: Optional[str] = "json") -> Node:
    """
    Decode a Stencila Schema node from a string

    :param str string: The string to decode to a node
    :param str format: The format to decode from
    :return: A Stencila Schema node
    """
    return from_json(await _stencila.convert.from_string(string, {"format": format}))


async def from_path(path: str, format: Optional[str] = None) -> Node:
    """
    Decode a Stencila Schema node from a filesystem path

    :param str path: The path to decode to a node
    :param str format: The format to decode from (if not supplied, inferred from the path)
    :return: A Stencila Schema node
    """
    return from_json(await _stencila.convert.from_path(path, {"format": format}))


async def to_string(
    node: Node,
    format: Optional[str] = "json",
    standalone: bool = False,
    compact: bool = False,
) -> str:
    """
    Encode a Stencila Schema node to a string

    :param Node node: The node to encode
    :param str format: The format to encode to
    :param bool standalone: Whether to encode as a standalone document
    :param bool compact: Whether to encode in compact form
    :return: The node encoded as a string in the format
    """
    return await _stencila.convert.to_string(
        to_json(node), {"format": format, "standalone": standalone, "compact": compact}
    )


async def to_path(
    node: Node,
    path: str,
    format: Optional[str] = None,
    standalone: bool = False,
    compact: bool = False,
):
    """
    Encode a Stencila Schema node to a filesystem path

    :param Node node: The node to encode
    :param str path: The path to encode the node to
    :param str format: The format to encode to (if not supplied, inferred from the path)
    :param bool standalone: Whether to encode as a standalone document
    :param bool compact: Whether to encode in compact form
    """
    return await _stencila.convert.to_path(
        to_json(node),
        path,
        {"format": format, "standalone": standalone, "compact": compact},
    )


async def from_to(
    input: Optional[str] = None,
    output: Optional[str] = None,
    from_format=None,
    to_format=None,
    to_standalone=False,
    to_compact=False,
) -> str:
    """
    Convert a document from one format to another

    :param str input: The input path (if not supplied, stdin will be read)
    :param str output: The output path (if not supplied, the converted input will be returned)
    :param str from_format: The format of the input (if not supplied, inferred from the input path)
    :param str to_format: The format of the output (if not supplied, inferred from the output path)
    :param bool standalone: Whether to encode as a standalone document
    :param bool compact: Whether to encode in compact form
    """
    return await _stencila.convert.from_to(
        input if input else "",
        output if output else "",
        {"format": from_format},
        {"format": to_format, "standalone": to_standalone, "compact": to_compact},
    )
