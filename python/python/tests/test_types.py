from stencila import shortcuts as S  # noqa: N812
from stencila import stencila_types as T  # noqa: N812
from stencila.utilities import from_json, to_json


def test_code():
    cb = T.CodeBlock(code="print('Hello world')", programming_language="python")
    cb_json = """
    {
        "type": "CodeBlock",
        "code": "print('Hello world')",
        "programming_language": "python"
    }
    """

    cb_parsed = from_json(cb_json)
    assert cb == cb_parsed

    out_json = to_json(cb)
    cb_roundtrip = from_json(out_json)
    assert cb_roundtrip == cb


def test_article():
    a_json = """
    {
        "type": "Article",
        "content": [
            {
                "type": "Paragraph",
                "content": [
                    {"type": "Text", "value": "Hi "},
                    {"type": "Text", "value": "There!"}
                ]
            }
        ]
    }
    """
    a_parsed = from_json(a_json)
    a_shortcut = S.art(content=S.p("Hi ", "There!"))
    assert a_parsed == a_shortcut

    out_json = to_json(a_shortcut)
    a_roundtrip = from_json(out_json)
    assert a_roundtrip == a_shortcut
