from typing import Optional

from stencila import _stencila
from stencila.types import Node
from stencila.utilities import to_json, from_json


async def from_string(string: str, format: str = "json") -> Node:
    return from_json(await _stencila.convert.from_string(string, {"format": format}))


async def from_path(path: str, format: str = "json") -> Node:
    return from_json(await _stencila.convert.from_path(path, {"format": format}))


async def to_string(
    node: Node, format: str = "json", standalone: bool = False, compact: bool = False
) -> str:
    return await _stencila.convert.to_string(
        to_json(node), {"format": format, "standalone": standalone, "compact": compact}
    )


async def to_path(
    node: Node,
    path: str,
    format: str = "json",
    standalone: bool = False,
    compact: bool = False,
):
    return await _stencila.convert.to_path(
        to_json(node),
        path,
        {"format": format, "standalone": standalone, "compact": compact},
    )


async def from_to(
    input: Optional[str] = None,
    output: Optional[str] = None,
    from_format="json",
    to_format="json",
    to_standalone=False,
    to_compact=False,
):
    return await _stencila.convert.from_to(
        input if input else "",
        output if output else "",
        {"format": from_format},
        {"format": to_format, "standalone": to_standalone, "compact": to_compact},
    )
