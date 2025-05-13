test.figure(@return).paragraph(.text ~= 'frogs')
---
MATCH (figure:Figure)-[:content|:items* acyclic]->(paragraph:Paragraph)
WHERE regexp_matches(paragraph.text, 'frogs')
RETURN figure
LIMIT 10
