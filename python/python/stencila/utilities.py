import json

from cattrs.preconf.json import make_converter

from stencila import stencila_types as T  # noqa: N812

JSON_CONVERTER = make_converter()


def from_json(json_string: str) -> T.Node:
    """
    Create a `Node` from a JSON string
    """
    dcts = json.loads(json_string)
    typ = dcts.get("type")
    if type is None:
        raise ValueError(
            f"JSON string does not contain a `type` attribute: {json_string}"
        )
    # TODO: Handle error
    cls = getattr(T, typ)
    return cls(**dcts)


# TODO: add more typing here. Should just be basic stuff + dict + list?
# def from_value(value: Any) -> T.Node:  # pragma: no cover
#     """
#     Create a `Node` from a value
#     """
#     # TODO: Handle Entity and tuple. When will this happen?
#     if value is None or isinstance(value, (bool, int, float, str)):
#         return value
#
#     if isinstance(value, list):
#         for index, item in enumerate(value):
#             value[index] = from_value(item)
#         return value
#
#     typ = value.pop("type", None)
#
#     # TODO: This should fail?
#     if typ is None:
#         return value
#
#     for attr in value:
#         value[attr] = from_value(value[attr])
#
#     try:
#         cls = getattr(T, typ)
#         return cls(**value)
#     except AttributeError:
#         raise ValueError(f"Unexpected type for `Node`: {typ}") from None


def to_json(node: T.Node) -> str:
    """
    Serialize a node to a JSON string
    """
    return JSON_CONVERTER.dumps(node)
