import json

from . import types
from .types import Node, Thing

# TODO: Use Entity instead of Thing
# When https://github.com/stencila/schema/issues/96 is done

def toDict(node: Thing) -> dict:
    """Convert a Thing node to a dictionary"""
    node_dict = {
        "type": node.__class__.__name__
    }
    node_dict.update(node.__dict__)
    return node_dict

def fromDict(node_dict: dict) -> Thing:
    """Convert a dictionary to a Thing node"""
    if "type" in node_dict:
        class_ = getattr(types, node_dict["type"])
        return class_(**node_dict)
    else:
        return node_dict

def toJSON(node: Node) -> str:
    """Convert a node to JSON"""
    return json.dumps(node, default=toDict, indent=2)

def fromJSON(json: str) -> Node:
    """Convert JSON to a Node"""
    node = json.loads(json)
    if isinstance(node, dict): return fromDict(node)
    else: return node
