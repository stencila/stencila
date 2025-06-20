paragraph()
---
MATCH (paragraph:Paragraph)
RETURN paragraph
LIMIT 1


paragraphs()
---
MATCH (paragraph:Paragraph)
RETURN paragraph
LIMIT 10


paragraph(above)
---
MATCH (paragraph:Paragraph)
WHERE paragraph.position<$currentPosition
RETURN paragraph
ORDER BY paragraph.position DESC
LIMIT 1


paragraph(below)
---
MATCH (paragraph:Paragraph)
WHERE paragraph.position>$currentPosition
RETURN paragraph
ORDER BY paragraph.position
LIMIT 1


paragraphs(above, skip = 1, limit = 2)
---
MATCH (paragraph:Paragraph)
WHERE paragraph.position<$currentPosition
RETURN paragraph
ORDER BY paragraph.position DESC
SKIP 1
LIMIT 2


paragraphs(above).skip(1).limit(1)
---
MATCH (paragraph:Paragraph)
WHERE paragraph.position<$currentPosition
RETURN paragraph
ORDER BY paragraph.position DESC
SKIP 1
LIMIT 1
