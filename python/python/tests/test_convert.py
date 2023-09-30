import tempfile

from stencila.convert import to_string, from_string, from_path, to_path, from_to
from stencila.types import Article, Paragraph, Text, Strong, Emphasis


async def test_from_string():
    node = await from_string(
        '{type: "Article", content: [{type: "Paragraph", content: ["Hello world"]}]}',
        format="json5",
    )

    assert isinstance(node, Article)
    assert isinstance(node.content[0], Paragraph)
    assert node == Article(content=[Paragraph(content=["Hello world"])])


async def test_from_path():
    node = await from_path("../examples/paragraph/paragraph.json")

    assert isinstance(node, Article)
    assert isinstance(node.content[0], Paragraph)
    assert node.content[0] == Paragraph(
        content=[Text(value="This is paragraph one. It has two sentences.")]
    )


async def test_to_string():
    markdown = await to_string(
        Article(
            content=[Paragraph(content=["Hello ", Strong(content=["world"]), "!"])]
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

    temp = tempfile.mktemp()
    await to_path(node, temp, format="jats", compact=True)
    round_tripped = await from_path(temp, format="jats")

    assert round_tripped == node


async def test_from_to():
    markdown = await from_to("../examples/paragraph/paragraph.json", to_format="md")

    assert (
        markdown
        == "This is paragraph one. It has two sentences.\n\nParagraph two, only has one sentence."
    )

    temp = tempfile.mktemp()
    await from_to(
        "../examples/paragraph/paragraph.json",
        temp,
        to_format="html",
        to_standalone=False,
        to_compact=True,
    )
    html = open(temp).read()

    assert (
        html
        == "<article><p><span>This is paragraph one. It has two sentences.</span><p><span>Paragraph two, only has one sentence.</span></article>"
    )
