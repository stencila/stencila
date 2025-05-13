test.paragraphs(like = 'some text')
---
CALL QUERY_VECTOR_INDEX('Paragraph', 'vector', $par1, 10)
RETURN node
ORDER BY distance
LIMIT 10
