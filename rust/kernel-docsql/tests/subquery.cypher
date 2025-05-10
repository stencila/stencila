test.articles(...authors(.name ^= 'Jane'))
---
MATCH (article:Article)
WHERE EXISTS { MATCH (article)-[authors]->(person:Person) WHERE starts_with(person.name, 'Jane') }
RETURN article
LIMIT 10


test.articles(.title ~= 'height', ...authors(.name ^= 'John').affiliations(.name $= 'University'))
---
MATCH (article:Article)
WHERE EXISTS { MATCH (article)-[authors]->(person:Person)-[:affiliations]->(org:Organization) WHERE starts_with(person.name, 'John') AND ends_with(org.name, 'University') }
  AND regexp_matches(article.title, 'height')
RETURN article
LIMIT 10


test.articles(...authors().gt(4))
---
MATCH (article:Article)
WHERE COUNT { MATCH (article)-[authors]->(person:Person) } > 4
RETURN article
LIMIT 10


test.articles(...refs().lte(10))
---
MATCH (article:Article)
WHERE COUNT { MATCH (article)-[references]->(ref:Reference) } <= 10
RETURN article
LIMIT 10
