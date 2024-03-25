import json
import re

from cattrs.preconf.json import make_converter

from stencila_types import types as T  # noqa: N812

JSON_CONVERTER = make_converter()


def from_json(json_string: str) -> T.Node:
    """
    Create a `Node` from a JSON string
    """
    # TODO: JSON_CONVERTER is not used here as it currently breaks.
    # We do it manually below.

    return from_value(json.loads(json_string))


# https://stackoverflow.com/questions/1175208/
# elegant-python-function-to-convert-camelcase-to-snake-case
CAMEL_TO_SNAKE_RE = re.compile("((?<=[a-z0-9])[A-Z]|(?!^)[A-Z](?=[a-z]))")


def camel_to_snake(name: str):
    return CAMEL_TO_SNAKE_RE.sub(r"_\1", name).lower()


def from_value(value) -> T.Node | list[T.Node]:  # pragma: no cover
    """
    Create a `Node` from a value
    """
    if value is None or isinstance(value, bool | int | float | str):
        return value

    # Handle lists.
    if isinstance(value, list):
        return [from_value(v) for v in value]

    # We should be a dictionary
    if not isinstance(value, dict):
        raise ValueError(f"Unexpected type for `Node`: {type(value)}")

    typ = value.pop("type", None)

    if typ is None:
        # We'll have to assume this is an "Object" type. i.e. Just a bare
        # dictionary in python.
        cls = dict
    else:
        cls = getattr(T, typ, None)
        if cls is None:
            raise ValueError(f"`{typ}` is not a valid Stencila Type")

    # Resolve the attributes
    # 1. Convert camelCase to snake_case a Stencila can store in different
    #    conventions.
    # 2. Remove any $schema attributes.
    kwargs = {
        camel_to_snake(nm): from_value(v)
        for (nm, v) in value.items()
        if nm != "$schema"
    }

    # value is a dictionary of attributes
    return cls(**kwargs)


def to_json(node: T.Node) -> str:
    """
    Serialize a node to a JSON string
    """
    return JSON_CONVERTER.dumps(node)
