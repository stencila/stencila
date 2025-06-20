introduction()
---
MATCH (section:Section)
WHERE section.sectionType = 'Introduction'
RETURN section
LIMIT 10


methods()
---
MATCH (section:Section)
WHERE section.sectionType = 'Methods'
RETURN section
LIMIT 10


results()
---
MATCH (section:Section)
WHERE section.sectionType = 'Results'
RETURN section
LIMIT 10


discussion()
---
MATCH (section:Section)
WHERE section.sectionType = 'Discussion'
RETURN section
LIMIT 10


introduction().paragraphs(return, limit = 2)
---
MATCH (section:Section)-[:content|:items* acyclic]->(paragraph:Paragraph)
WHERE section.sectionType = 'Introduction'
RETURN paragraph
LIMIT 2
