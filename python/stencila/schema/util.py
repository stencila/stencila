"""Utility functions."""


def node_type(node) -> str:
    """
    Get the type of the node.

    This is the Python equivalent of the
    [`nodeType`](https://github.com/stencila/schema/blob/bd90c808d14136c8489ce8bb945b2bb6085b9356/ts/util/nodeType.ts)
    function.
    """
    # pylint: disable=R0911

    if node is None:
        return "Null"
    if isinstance(node, bool):
        return "Boolean"
    if isinstance(node, (int, float)):
        return "Number"
    if isinstance(node, str):
        return "Text"
    if isinstance(node, (list, tuple)):
        return "Array"
    if isinstance(node, dict):
        type_name = node.get("type")
        if type_name is not None:
            return type_name
    return "Object"
