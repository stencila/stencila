import json

from stencila.types import *

# The above 'import *' imports `types/list.py` as list so
# it is necessary to import it again from builtins.
# This may be able to be removed later.
from builtins import list


def from_json(json_string: str) -> Node:
    """
    Create a `Node` from a JSON string
    """
    import json

    return from_value(json.loads(json_string))


def from_value(value) -> Node:  # pragma: no cover
    """
    Create a `Node` from a value
    """
    if (
        value is None
        or isinstance(value, bool)
        or isinstance(value, int)
        or isinstance(value, float)
        or isinstance(value, str)
        or isinstance(value, tuple)
        or isinstance(value, Entity)
    ):
        return value

    if isinstance(value, list):
        for index, item in enumerate(value):
            value[index] = from_value(item)
        return value

    typ = value.get("type")

    if typ is None:
        return value

    del value["type"]

    for attr in value.keys():
        value[attr] = from_value(value[attr])

    if typ == "Admonition":
        return Admonition(**value)
    if typ == "ArrayValidator":
        return ArrayValidator(**value)
    if typ == "Article":
        return Article(**value)
    if typ == "AudioObject":
        return AudioObject(**value)
    if typ == "BooleanValidator":
        return BooleanValidator(**value)
    if typ == "Brand":
        return Brand(**value)
    if typ == "Button":
        return Button(**value)
    if typ == "CallArgument":
        return CallArgument(**value)
    if typ == "CallBlock":
        return CallBlock(**value)
    if typ == "Cite":
        return Cite(**value)
    if typ == "CiteGroup":
        return CiteGroup(**value)
    if typ == "Claim":
        return Claim(**value)
    if typ == "CodeBlock":
        return CodeBlock(**value)
    if typ == "CodeChunk":
        return CodeChunk(**value)
    if typ == "CodeExpression":
        return CodeExpression(**value)
    if typ == "CodeInline":
        return CodeInline(**value)
    if typ == "Collection":
        return Collection(**value)
    if typ == "Comment":
        return Comment(**value)
    if typ == "ConstantValidator":
        return ConstantValidator(**value)
    if typ == "ContactPoint":
        return ContactPoint(**value)
    if typ == "CreativeWork":
        return CreativeWork(**value)
    if typ == "Datatable":
        return Datatable(**value)
    if typ == "DatatableColumn":
        return DatatableColumn(**value)
    if typ == "Date":
        return Date(**value)
    if typ == "DateTime":
        return DateTime(**value)
    if typ == "DateTimeValidator":
        return DateTimeValidator(**value)
    if typ == "DateValidator":
        return DateValidator(**value)
    if typ == "DefinedTerm":
        return DefinedTerm(**value)
    if typ == "Delete":
        return Delete(**value)
    if typ == "Directory":
        return Directory(**value)
    if typ == "Duration":
        return Duration(**value)
    if typ == "DurationValidator":
        return DurationValidator(**value)
    if typ == "Emphasis":
        return Emphasis(**value)
    if typ == "EnumValidator":
        return EnumValidator(**value)
    if typ == "Enumeration":
        return Enumeration(**value)
    if typ == "ExecutionDependant":
        return ExecutionDependant(**value)
    if typ == "ExecutionDependency":
        return ExecutionDependency(**value)
    if typ == "ExecutionDigest":
        return ExecutionDigest(**value)
    if typ == "ExecutionError":
        return ExecutionError(**value)
    if typ == "ExecutionTag":
        return ExecutionTag(**value)
    if typ == "Figure":
        return Figure(**value)
    if typ == "File":
        return File(**value)
    if typ == "ForBlock":
        return ForBlock(**value)
    if typ == "Form":
        return Form(**value)
    if typ == "Function":
        return Function(**value)
    if typ == "Grant":
        return Grant(**value)
    if typ == "Heading":
        return Heading(**value)
    if typ == "IfBlock":
        return IfBlock(**value)
    if typ == "IfBlockClause":
        return IfBlockClause(**value)
    if typ == "ImageObject":
        return ImageObject(**value)
    if typ == "IncludeBlock":
        return IncludeBlock(**value)
    if typ == "Insert":
        return Insert(**value)
    if typ == "IntegerValidator":
        return IntegerValidator(**value)
    if typ == "Link":
        return Link(**value)
    if typ == "List":
        return List(**value)
    if typ == "ListItem":
        return ListItem(**value)
    if typ == "MathBlock":
        return MathBlock(**value)
    if typ == "MathInline":
        return MathInline(**value)
    if typ == "MediaObject":
        return MediaObject(**value)
    if typ == "MonetaryGrant":
        return MonetaryGrant(**value)
    if typ == "Note":
        return Note(**value)
    if typ == "NumberValidator":
        return NumberValidator(**value)
    if typ == "Organization":
        return Organization(**value)
    if typ == "Paragraph":
        return Paragraph(**value)
    if typ == "Parameter":
        return Parameter(**value)
    if typ == "Periodical":
        return Periodical(**value)
    if typ == "Person":
        return Person(**value)
    if typ == "PostalAddress":
        return PostalAddress(**value)
    if typ == "Product":
        return Product(**value)
    if typ == "PropertyValue":
        return PropertyValue(**value)
    if typ == "PublicationIssue":
        return PublicationIssue(**value)
    if typ == "PublicationVolume":
        return PublicationVolume(**value)
    if typ == "QuoteBlock":
        return QuoteBlock(**value)
    if typ == "QuoteInline":
        return QuoteInline(**value)
    if typ == "Review":
        return Review(**value)
    if typ == "SoftwareApplication":
        return SoftwareApplication(**value)
    if typ == "SoftwareSourceCode":
        return SoftwareSourceCode(**value)
    if typ == "Strikeout":
        return Strikeout(**value)
    if typ == "StringValidator":
        return StringValidator(**value)
    if typ == "Strong":
        return Strong(**value)
    if typ == "StyledBlock":
        return StyledBlock(**value)
    if typ == "StyledInline":
        return StyledInline(**value)
    if typ == "Subscript":
        return Subscript(**value)
    if typ == "Superscript":
        return Superscript(**value)
    if typ == "Table":
        return Table(**value)
    if typ == "TableCell":
        return TableCell(**value)
    if typ == "TableRow":
        return TableRow(**value)
    if typ == "Text":
        return Text(**value)
    if typ == "ThematicBreak":
        return ThematicBreak(**value)
    if typ == "Thing":
        return Thing(**value)
    if typ == "Time":
        return Time(**value)
    if typ == "TimeValidator":
        return TimeValidator(**value)
    if typ == "Timestamp":
        return Timestamp(**value)
    if typ == "TimestampValidator":
        return TimestampValidator(**value)
    if typ == "TupleValidator":
        return TupleValidator(**value)
    if typ == "Underline":
        return Underline(**value)
    if typ == "Variable":
        return Variable(**value)
    if typ == "VideoObject":
        return VideoObject(**value)

    raise ValueError(f"Unexpected type for `Node`: {typ}")


def to_json(node: Node) -> str:
    """
    Serialize a node to a JSON string
    """
    return node.to_json() if isinstance(node, Entity) else json.dumps(node)
