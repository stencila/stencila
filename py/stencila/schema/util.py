"""Utility functions for schema to/from JSON."""

import json
import typing

from . import types
from .types import Node, Entity


def to_dict(node: Entity) -> dict:
    """Convert an Entity node to a dictionary"""
    node_dict = {
        "type": node.__class__.__name__
    }
    node_dict.update(node.__dict__)
    return node_dict


def from_dict(node_dict: dict) -> typing.Union[dict, Node]:
    """Convert a dictionary to an Entity node (if it has a `type` item)."""
    if "type" in node_dict:
        node_type = node_dict.pop("type")
        class_ = getattr(types, node_type)
        return class_(**node_dict)
    return node_dict


def to_json(node: Node) -> str:
    """Convert a node to JSON"""
    return json.dumps(node, default=to_dict, indent=2)


def from_json(serialized: str) -> typing.Union[dict, Node]:
    """Convert JSON to a Node"""
    node = json.loads(serialized)
    return from_dict(node) if isinstance(node, dict) else node
