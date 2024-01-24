"""
Tests of functions in the `convert` module
"""

from tempfile import NamedTemporaryFile

import pytest

from stencila.convert import to_string, from_string, from_path, to_path, from_to
from stencila.types import Article, Paragraph, Text, Strong, Emphasis


async def test_from_string():
    node = await from_string(
        '{type: "Article", content: [{type: "Paragraph", content: [{type: "Text", value: "Hello world"}]}]}',
        format="json5",
    )

    assert isinstance(node, Article)
    assert isinstance(node.content[0], Paragraph)
    assert node == Article([Paragraph([Text("Hello world")])])


@pytest.mark.skip(reason="currently failing due to casing of field in constructor")
async def test_from_path():
    node = await from_path("../examples/nodes/paragraph/paragraph.json")

    assert isinstance(node, Article)
    assert isinstance(node.content[0], Paragraph)
    assert node.content[0] == Paragraph(
        [Text("This is paragraph one. It has two sentences.")]
    )


async def test_to_string():
    markdown = await to_string(
        Article(
            [
                Paragraph(
                    [
                        Text("Hello "),
                        Strong([Text("world")]),
                        Text("!"),
                    ]
                )
            ]
        ),
        format="md",
    )

    assert markdown == "Hello **world**!"


async def test_to_path():
    node = Article(
        [
            Paragraph(
                [
                    Text("Hello file "),
                    Emphasis([Text("system")]),
                    Text("!"),
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
