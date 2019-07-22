from python.types import Article, Person, Paragraph

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

