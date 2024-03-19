from stencila_types import shortcuts as S
from stencila_types import types as T
from stencila_types.utilities import from_json, to_json


def test_code_block():
    cb = T.CodeBlock(code="print('Hello world')", programming_language="python")

    assert cb.type == "CodeBlock"
    assert cb.code == "print('Hello world')"
    assert cb.authors is None
    assert cb.programming_language == "python"


def test_paragraph():
    p1 = T.Paragraph(content=[T.Text(value="Hi "), T.Text(value="There!")])
    p2 = S.p(["Hi ", "There!"])
    assert p1 == p2

    p3 = S.p("Hi ", "There!")
    assert p1 == p3

    # TODO: SHould be in shortcuts.
    p4 = S.p(1, 2, [2, 4])
    print(p4)


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
    # TODO: Fix p so it takes *args too.
    a_shortcut = S.art(content=S.p(["Hi ", "There!"]))
    assert a_parsed == a_shortcut

    out_json = to_json(a_shortcut)
    a_roundtrip = from_json(out_json)
    assert a_roundtrip == a_shortcut
