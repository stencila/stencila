def test_article_example():
    """Test to check that README example runs"""

    from stencila.schema.types import Article, Person
    from stencila.schema.json import encode

    encode(
        Article(
            title="The Impact of Interactive Epistemologies on Cryptography",
            authors=[
                Person(givenNames=["Josiah", "Stinkney"], familyNames=["Carberry"])
            ],
        )
    )
