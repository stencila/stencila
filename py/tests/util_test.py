import json
import typing

from stencila.schema import types
from stencila.schema.util import to_json, from_json


def test_to_json():
    """Test that a large schema transforms to json OK (recursion works)."""
    article = types.Article(
        title='My Article',
        authors=[types.Person(
            givenNames=['John'],
            familyNames=['Doe']
        )],
        content=[
            types.Heading(content=['Title'], depth=1),
            types.Paragraph(content=['This is the opening paragraph.']),
            types.Paragraph(content=[
                'This contains two lists.',
                types.List(items=[
                    types.ListItem(content=['Item One'], checked=True),
                    types.ListItem(content=['Item Two'], checked=True),
                    types.ListItem(content=['Item Three'], checked=True),
                ]),
                types.List(items=[
                    types.ListItem(content=['Item One'], checked=True),
                    types.ListItem(content=['Item Two'], checked=True),
                    types.ListItem(content=['Item Three'], checked=True),
                ])
            ])
        ]
    )

    # Go back and forth via JSON to compare to dict – means we don't have to worry about differences in spacing etc
    # in the generated JSON string
    assert json.loads(to_json(article)) == {
        'type': 'Article',
        'authors': [
            {
                'type': 'Person',
                'familyNames': [
                    'Doe'
                ],
                'givenNames': [
                    'John'
                ]
            }
        ],
        'content': [
            {
                'type': 'Heading',
                'content': [
                    'Title'
                ],
                'depth': 1
            },
            {
                'type': 'Paragraph',
                'content': [
                    'This is the opening paragraph.'
                ]
            },
            {
                'type': 'Paragraph',
                'content': [
                    'This contains two lists.',
                    {
                        'type': 'List',
                        'items': [
                            {
                                'type': 'ListItem',
                                'content': [
                                    'Item One'
                                ],
                                'checked': True
                            },
                            {
                                'type': 'ListItem',
                                'content': [
                                    'Item Two'
                                ],
                                'checked': True
                            },
                            {
                                'type': 'ListItem',
                                'content': [
                                    'Item Three'
                                ],
                                'checked': True
                            }
                        ]
                    },
                    {
                        'type': 'List',
                        'items': [
                            {
                                'type': 'ListItem',
                                'content': [
                                    'Item One'
                                ],
                                'checked': True
                            },
                            {
                                'type': 'ListItem',
                                'content': [
                                    'Item Two'
                                ],
                                'checked': True
                            },
                            {
                                'type': 'ListItem',
                                'content': [
                                    'Item Three'
                                ],
                                'checked': True
                            }
                        ]
                    }
                ]
            }
        ],
        'title': 'My Article'
    }


def test_from_json():
    """Test unserializing JSON back to native Python objects."""
    article = from_json("""
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
    """)

    article = typing.cast(types.Article, article)

    assert isinstance(article, types.Article)
    assert article.title == 'Article Title'

    assert len(article.authors) == 1
    assert isinstance(article.authors[0], types.Person)
    assert article.authors[0].givenNames == ['Jane']
    assert article.authors[0].familyNames == ['Dorn']

    assert len(article.content) == 2

    assert isinstance(article.content[0], types.Heading)
    assert article.content[0].content == ['Heading']
    assert article.content[0].depth == 2

    assert isinstance(article.content[1], types.Paragraph)

    assert len(article.content[1].content) == 2
    assert article.content[1].content[0] == 'Some text '

    assert isinstance(article.content[1].content[1], types.Link)
    assert article.content[1].content[1].target == 'http://www.example.com'

    assert len(article.content[1].content[1].content) == 1
    assert article.content[1].content[1].content[0] == 'With a Link'
