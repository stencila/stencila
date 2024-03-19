"""
Tests of functions in the `convert` module
"""

from tempfile import NamedTemporaryFile

from stencila import shortcuts as S  # noqa: N812
from stencila.convert import from_path, from_string, from_to, to_path, to_string
from stencila.stencila_types import Article, Emphasis, Paragraph, Strong, Text


async def test_from_string():
    txt = """{
        type: "Article",
        content: [
                {type: "Paragraph", content: [{type: "Text", value: "Hello world"}]}
            ]
        }
    """

    node = await from_string(txt, format="json5")
    assert isinstance(node, Article)
    assert isinstance(node.content[0], Paragraph)

    # Should be the same.
    a = S.art(S.p("Hello world"))
    assert node == a


async def test_from_path():
    node = await from_path("../examples/nodes/paragraph/paragraph.json")

    assert isinstance(node, Article)
    assert isinstance(node.content[0], Paragraph)
    assert node.content[0] == Paragraph(
        content=[Text(value="This is paragraph one. It has two sentences.")]
    )


async def test_to_string():
    markdown = await to_string(
        Article(
            content=[
                Paragraph(
                    content=[
                        Text(value="Hello "),
                        Strong(content=[Text(value="world")]),
                        Text(value="!"),
                    ]
                )
            ]
        ),
        format="md",
    )

    assert markdown == "Hello **world**!"


async def test_to_path():
    node = Article(
        content=[
            Paragraph(
                content=[
                    Text(value="Hello file "),
                    Emphasis(content=[Text(value="system")]),
                    Text(value="!"),
                ]
            ),
        ]
    )

    with NamedTemporaryFile(mode="w+", delete=False) as temp:
        await to_path(node, temp.name, format="jats", compact=True)
        round_tripped = await from_path(temp.name, format="jats")
        assert round_tripped == node


async def test_from_to():
    markdown = await from_to(
        "../examples/nodes/paragraph/paragraph.json", to_format="md"
    )

    assert markdown.startswith("This is paragraph one. It has two sentences.")

    with NamedTemporaryFile(mode="w+", delete=False) as temp:
        await from_to(
            "../examples/nodes/paragraph/paragraph.json",
            temp.name,
            to_format="html",
            to_standalone=False,
            to_compact=True,
        )
        html = open(temp.name).read()
        assert html.startswith(
            "<article><p><span>This is paragraph one. It has two sentences."
        )
