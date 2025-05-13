test.paragraphs(search = 'keyword')
---
CALL QUERY_FTS_INDEX('Paragraph', 'fts', 'keyword')
RETURN node
LIMIT 10


test.paragraphs(searchAll = 'keyword1 keyword2')
---
CALL QUERY_FTS_INDEX('Paragraph', 'fts', 'keyword1 keyword2', conjunctive := true)
RETURN node
LIMIT 10


test.paragraphs(.text ^= 'Word', search = 'keyword')
---
CALL QUERY_FTS_INDEX('Paragraph', 'fts', 'keyword')
WHERE starts_with(node.text, 'Word')
RETURN node
LIMIT 10
