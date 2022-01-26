"""
Functions for converting nodes to/from JSON.

This is a JSON "codec" analogous to that in
[Encoda](https://github.com/stencila/encoda/blob/v0.85.3/src/codecs/json/index.ts)
which provides `encode` and `decode` functions to/from JSON and Python
objects representing Schema nodes.
"""
import json
import typing

from . import types
from .types import Node, Entity


def decode(serialized: str) -> Node:
    """Decode JSON as a `Node`"""
    node = json.loads(serialized)
    return dict_decode(node) if isinstance(node, dict) else node


def encode(node: Node) -> str:
    """Encode a `Node` to JSON"""
    return json.dumps(node, default=object_encode, indent=2)


def dict_decode(node_dict: dict) -> Node:
    """Convert a dictionary to an `Entity` node (if it has a `type` item)."""
    if "type" not in node_dict:
        return node_dict

    node_type = node_dict.pop("type")
    class_ = getattr(types, node_type, None)

    if class_ is None:
        return node_dict

    node_kwargs = {}

    for key, val in node_dict.items():
        if isinstance(val, dict):
            val = dict_decode(val)
        elif isinstance(val, list):
            processed_list = []
            for sub_val in val:
                if isinstance(sub_val, dict):
                    processed_list.append(dict_decode(sub_val))
                else:
                    processed_list.append(sub_val)
            val = processed_list

        node_kwargs[key] = val

    return class_(**node_kwargs)


def object_encode(node: typing.Any) -> typing.Union[dict, str]:
    """Convert an `Entity` node to a dictionary"""
    if not isinstance(node, Entity):
        return str(node)

    node_dict = {"type": node.__class__.__name__}
    node_dict.update(node.__dict__)
    return node_dict
