from schema.types import Article, Person, Paragraph
from schema.util import to_json

article = Article(
    title='',
    authors=[
        Person(
            givenNames=['Jane']
        ),
    ],
    content=[
        Paragraph(['Hello'])
    ]
)

print(to_json(article))
