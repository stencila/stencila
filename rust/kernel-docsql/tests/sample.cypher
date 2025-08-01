test.paragraphs().sample()
---
MATCH (paragraph:Paragraph)
RETURN paragraph
ORDER BY gen_random_uuid()
LIMIT 10


test.paragraphs().sample(20)
---
MATCH (paragraph:Paragraph)
RETURN paragraph
ORDER BY gen_random_uuid()
LIMIT 20
