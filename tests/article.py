from python.types import Article, Person, Paragraph
from python.util import toJSON

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

print(toJSON(article))
