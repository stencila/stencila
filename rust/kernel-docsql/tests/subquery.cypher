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


test.articles(...authors(* > 4))
---
MATCH (article:Article)
WHERE COUNT { MATCH (article)-[authors]->(person:Person) } > 4
RETURN article
LIMIT 10


test.articles(...references(* <= 10))
---
MATCH (article:Article)
WHERE COUNT { MATCH (article)-[references]->(ref:Reference) } <= 10
RETURN article
LIMIT 10


test.articles(...references(* == 1))
---
MATCH (article:Article)
WHERE COUNT { MATCH (article)-[references]->(ref:Reference) } = 1
RETURN article
LIMIT 10


test.articles(...references(* in [1,2,3]))
---
MATCH (article:Article)
WHERE COUNT { MATCH (article)-[references]->(ref:Reference) } IN [1, 2, 3]
RETURN article
LIMIT 10


test.articles(...references(* ~= 1))
---
only numeric comparison operators (e.g. <=) can be used in count filters (*)


test.articles(...references(* > 1, * < 10))
---
only one count filter (*) allowed per call


test.articles(* < 10)
---
count filters (*) can only be used with subqueries
