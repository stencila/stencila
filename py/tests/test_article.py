def test_article_example():
    """Test to check that README example runs"""

    from stencila.schema.types import Article, Person
    from stencila.schema.util import to_json

    to_json(
        Article(
            title = "The Impact of Interactive Epistemologies on Cryptography",
            authors = [
                Person(
                    givenNames = ["Josiah", "Stinkney"],
                    familyNames = ["Carberry"]
                )
            ]
        )
    )
