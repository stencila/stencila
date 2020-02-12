import json
import typing

from stencila.schema.types import (
    Article,
    Person,
    Heading,
    Paragraph,
    List,
    ListItem,
    Link,
)
from stencila.schema.json import encode, decode, dict_decode


def test_encode():
    """Test that a large schema transforms to json OK (recursion works)."""
    article = Article(
        title="My Article",
        authors=[Person(givenNames=["John"], familyNames=["Doe"])],
        content=[
            Heading(content=["Title"], depth=1),
            Paragraph(content=["This is the opening paragraph."]),
            Paragraph(
                content=[
                    "This contains two lists.",
                    List(
                        items=[
                            ListItem(content=["Item One"], checked=True),
                            ListItem(content=["Item Two"], checked=True),
                            ListItem(content=["Item Three"], checked=True),
                        ]
                    ),
                    List(
                        items=[
                            ListItem(content=["Item One"], checked=True),
                            ListItem(content=["Item Two"], checked=True),
                            ListItem(content=["Item Three"], checked=True),
                        ]
                    ),
                ]
            ),
        ],
    )

    # Go back and forth via JSON to compare to dict – means we don't have to worry about differences in spacing etc
    # in the generated JSON string
    assert json.loads(encode(article)) == {
        "type": "Article",
        "authors": [{"type": "Person", "familyNames": ["Doe"], "givenNames": ["John"]}],
        "content": [
            {"type": "Heading", "content": ["Title"], "depth": 1},
            {"type": "Paragraph", "content": ["This is the opening paragraph."]},
            {
                "type": "Paragraph",
                "content": [
                    "This contains two lists.",
                    {
                        "type": "List",
                        "items": [
                            {
                                "type": "ListItem",
                                "content": ["Item One"],
                                "checked": True,
                            },
                            {
                                "type": "ListItem",
                                "content": ["Item Two"],
                                "checked": True,
                            },
                            {
                                "type": "ListItem",
                                "content": ["Item Three"],
                                "checked": True,
                            },
                        ],
                    },
                    {
                        "type": "List",
                        "items": [
                            {
                                "type": "ListItem",
                                "content": ["Item One"],
                                "checked": True,
                            },
                            {
                                "type": "ListItem",
                                "content": ["Item Two"],
                                "checked": True,
                            },
                            {
                                "type": "ListItem",
                                "content": ["Item Three"],
                                "checked": True,
                            },
                        ],
                    },
                ],
            },
        ],
        "title": "My Article",
    }


def test_decode():
    """Test unserializing JSON back to native Python objects."""
    article = decode(
        """
        {
            "type": "Article",
            "title": "Article Title",
            "authors": [
                {
                    "type": "Person",
                    "givenNames": ["Jane"],
                    "familyNames": ["Dorn"]
                }
            ],
            "content": [
                {"type": "Heading", "content": ["Heading"], "depth": 2},
                {
                    "type": "Paragraph",
                    "content": [
                        "Some text ",
                        {
                            "type": "Link",
                            "target": "http://www.example.com",
                            "content": ["With a Link"],
                            "meta": {"dict": true}
                        }
                    ]
                }
            ]
        }
    """
    )

    article = typing.cast(Article, article)

    assert isinstance(article, Article)
    assert article.title == "Article Title"

    assert len(article.authors) == 1
    assert isinstance(article.authors[0], Person)
    assert article.authors[0].givenNames == ["Jane"]
    assert article.authors[0].familyNames == ["Dorn"]

    assert len(article.content) == 2

    assert isinstance(article.content[0], Heading)
    assert article.content[0].content == ["Heading"]
    assert article.content[0].depth == 2

    assert isinstance(article.content[1], Paragraph)

    assert len(article.content[1].content) == 2
    assert article.content[1].content[0] == "Some text "

    assert isinstance(article.content[1].content[1], Link)
    assert article.content[1].content[1].target == "http://www.example.com"

    assert len(article.content[1].content[1].content) == 1
    assert article.content[1].content[1].content[0] == "With a Link"
    assert article.content[1].content[1].meta == {"dict": True}


def test_dict_decode_with_bad_type():
    """If calling `dict_decode` with a dict that refers to a type that does not exist, just return the dict.."""
    no_type = {"type": "NoSuchTypeShouldEverExist", "a": 1, "b": 2}

    assert no_type is dict_decode(no_type)
