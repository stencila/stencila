test.paragraphs().skip(5)
---
MATCH (paragraph:Paragraph)
RETURN paragraph
SKIP 5
LIMIT 10


test.paragraphs().limit(3)
---
MATCH (paragraph:Paragraph)
RETURN paragraph
LIMIT 3


test.paragraphs().skip(5).limit(5)
---
MATCH (paragraph:Paragraph)
RETURN paragraph
SKIP 5
LIMIT 5


test.paragraphs().limit(5).skip(10)
---
MATCH (paragraph:Paragraph)
RETURN paragraph
SKIP 10
LIMIT 5


test.paragraphs().skip(10).limit(3)
---
MATCH (paragraph:Paragraph)
RETURN paragraph
SKIP 10
LIMIT 3
