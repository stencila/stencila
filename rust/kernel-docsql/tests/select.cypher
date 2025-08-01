test.articles().select('title', 'doi')
---
MATCH (article:Article)
RETURN article.title AS `title`, article.doi AS `doi`
