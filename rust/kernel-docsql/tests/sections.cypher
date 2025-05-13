test.abstracts()
---
MATCH (section:Section)
WHERE section.sectionType = 'Abstract'
RETURN section
LIMIT 10


test.introductions()
---
MATCH (section:Section)
WHERE section.sectionType = 'Introduction'
RETURN section
LIMIT 10


test.methods()
---
MATCH (section:Section)
WHERE section.sectionType = 'Methods'
RETURN section
LIMIT 10


test.results()
---
MATCH (section:Section)
WHERE section.sectionType = 'Results'
RETURN section
LIMIT 10


test.discussions()
---
MATCH (section:Section)
WHERE section.sectionType = 'Discussion'
RETURN section
LIMIT 10
