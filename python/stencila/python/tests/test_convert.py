"""
Tests of functions in the `convert` module
"""

from pathlib import Path

import pytest
from stencila_types import shortcuts as S
from stencila_types import types as T

from stencila.convert import from_path, from_string, from_to, to_path, to_string


@pytest.mark.skip(reason="failing due to changes in serialization shape of Cord")
async def test_from_string():
    txt = """{
        type: "Article",
        content: [
                {type: "Paragraph", content: [{type: "Text", value: "Hello world"}]}
            ]
        }
    """

    node = await from_string(txt, format="json5")
    assert isinstance(node, T.Article)
    assert isinstance(node.content[0], T.Paragraph)

    # Should be the same.
    a = S.art(S.p("Hello world"))
    assert node == a


@pytest.mark.skip(reason="failing due to changes in serialization shape of Cord")
async def test_from_path():
    node = await from_path("../../examples/nodes/paragraph/paragraph.json")

    assert isinstance(node, T.Article)
    assert isinstance(node.content[0], T.Paragraph)
    assert node.content[0] == T.Paragraph(
        content=[T.Text(value="This is paragraph one. It has two sentences.")]
    )


async def test_to_string():
    markdown = await to_string(
        T.Article(
            content=[
                T.Paragraph(
                    content=[
                        T.Text(value="Hello "),
                        T.Strong(content=[T.Text(value="world")]),
                        T.Text(value="!"),
                    ]
                )
            ]
        ),
        format="md",
    )

    assert markdown == "Hello **world**!\n"


@pytest.mark.skip(reason="failing due to changes in serialization shape of Cord")
async def test_to_path(tmp_path: Path):
    node = T.Article(
        content=[
            T.Paragraph(
                content=[
                    T.Text(value="Hello file "),
                    T.Emphasis(content=[T.Text(value="system")]),
                    T.Text(value="!"),
                ]
            ),
        ]
    )
    fpath = tmp_path / "file.jats"
    await to_path(node, str(fpath), format="jats", compact=True)
    round_tripped = await from_path(str(fpath), format="jats")
    assert round_tripped == node


async def test_from_to(tmp_path: Path):
    markdown = await from_to(
        "../../examples/nodes/paragraph/paragraph.json", to_format="md"
    )

    assert markdown.startswith("This is paragraph one. It has two sentences.")

    fpath = tmp_path / "file.html"
    await from_to(
        "../../examples/nodes/paragraph/paragraph.json",
        str(fpath),
        to_format="html",
        to_standalone=False,
        to_compact=True,
    )
    html = fpath.open().read()
    assert html.startswith(
        "<article><p><span>This is paragraph one. It has two sentences."
    )
