import json
from typing import Any

from stencila import types as T  # noqa: N812


def from_json(json_string: str) -> T.Node:
    """
    Create a `Node` from a JSON string
    """
    import json

    return from_value(json.loads(json_string))


# TODO: add more typing here. Should just be basic stuff + dict + list?
def from_value(value: Any) -> T.Node:  # pragma: no cover
    """
    Create a `Node` from a value
    """
    if value is None or isinstance(value, (bool, int, float, str, tuple, T.Entity)):
        return value

    if isinstance(value, list):
        for index, item in enumerate(value):
            value[index] = from_value(item)
        return value

    typ = value.pop("type", None)

    if typ is None:
        return value

    for attr in value:
        value[attr] = from_value(value[attr])

    try:
        cls = getattr(T, typ)
        return cls(**value)
    except AttributeError:
        raise ValueError(f"Unexpected type for `Node`: {typ}") from None


def to_json(node: T.Node) -> str:
    """
    Serialize a node to a JSON string
    """
    return node.to_json() if isinstance(node, T.Entity) else json.dumps(node)
