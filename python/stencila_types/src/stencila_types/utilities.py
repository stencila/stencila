# SPDX-FileCopyrightText: 2024 Nokome Bentley
#
# SPDX-License-Identifier: Apache-2.0

import json
import re
from typing import Any

from cattrs import Converter
from cattrs.preconf.json import make_converter

from stencila_types import types as T  # noqa: N812

JSON_CONVERTER = make_converter()


def to_json(node: Any) -> str:
    """
    Serialize a node to a JSON string

    The basic converter is unproblematic.
    """
    return JSON_CONVERTER.dumps(node)


def make_stencila_converter():
    """
    Specialise a converter for structuring Stencila nodes

    If we find any Stencila nodes or unions containing stencila nodes, we defer
    to our own `from_value` function to convert them. This means we can also
    handle types that wrap stencila nodes, letting cattrs deal with them. This
    is required for some of the types that we use in the APIs that wrap
    stencila nodes.

    https://catt.rs/en/stable/unions.html
    """

    converter = Converter()

    for u in T.UNIONS:
        converter.register_structure_hook(u, lambda o, _: from_value(o))

    for u in T.ANON_UNIONS:
        # Skip unions that don't have any stencila types
        for node in u.__args__:
            if hasattr(node, "type"):
                break
        else:
            continue

        converter.register_structure_hook(u, lambda o, _: from_value(o))

    for t in T.TYPES:
        converter.register_structure_hook(t, lambda o, _: from_value(o))

    return converter


STENCILA_CONVERTER = make_stencila_converter()


def from_json(json_string: str) -> T.Node:
    """
    Create a `Node` from a JSON string
    """
    return STENCILA_CONVERTER.structure(json.loads(json_string), T.Node)  # type: ignore


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
